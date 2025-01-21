use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::dealloc;
use error::{PagingError, VmmError};
use mem::{
    heap::align_up,
    paging::{ptm::PageTableManager, PageEntryFlags},
    VirtualAddress, PAGE_SIZE, VMM_PAGE_COUNT, VMM_VIRTUAL,
};
use object::{VmFlags, VmObject};
use paging::PTM;
use sync::locked::Locked;

pub(crate) mod error;
pub(crate) mod object;
pub(crate) mod paging;

pub(crate) static VMM: Locked<VirtualMemoryManager> = Locked::new();

/// Initializes the global virtual memory manager.
///
/// # Safety
/// This consumes the global page table manager, it thus cannot be used directly anymore after this
/// function call.
pub unsafe fn initialize() -> Result<(), VmmError> {
    let mut plocked = PTM.locked();
    let ptm = plocked
        .take()
        .ok_or(VmmError::Paging(PagingError::PtmUnitialized))?;

    let vlocked = VMM.locked();
    vlocked.get_or_init(|| unsafe { VirtualMemoryManager::new(VMM_VIRTUAL, VMM_PAGE_COUNT, ptm) });
    Ok(())
}

/// Uses page table manager and kernel heap to keep track of allocated virtual memory objects with specific permissions.
#[derive(Debug)]
pub(crate) struct VirtualMemoryManager {
    head: Option<NonNull<VmObject>>,
    vmm_start: VirtualAddress,
    vmm_page_count: usize,
    pages_allocated: usize,
    ptm: PageTableManager,
}

impl VirtualMemoryManager {
    /// Create a new VMM.
    ///
    /// # Safety
    /// Caller must ensure that the start address and page count are both valid and that the
    /// addres-srange is mapped. The VMM must only be used after the heap has been initialized.
    pub(crate) unsafe fn new(
        vmm_start: VirtualAddress,
        vmm_page_count: usize,
        ptm: PageTableManager,
    ) -> Self {
        Self {
            vmm_start,
            vmm_page_count,
            head: None,
            pages_allocated: 0,
            ptm,
        }
    }
}

impl VirtualMemoryManager {
    pub(crate) fn ptm(&mut self) -> &mut PageTableManager {
        &mut self.ptm
    }
}

impl VirtualMemoryManager {
    /// Allocates a new virtual memory object according to the length in bytes and other given arguments.
    pub(crate) fn alloc(
        &mut self,
        length: usize,
        flags: VmFlags,
        allocation_type: AllocationType,
    ) -> Result<VirtualAddress, VmmError> {
        // align length to next valid page size
        let length = align_up(length as u64, PAGE_SIZE) as usize;
        let mut base = 0;
        let mut current = self.head;

        // check if there is enough space for vmm object
        _ = self
            .pages_allocated
            .checked_add(length / PAGE_SIZE)
            .map(|res| res > self.vmm_page_count)
            .ok_or(VmmError::Oom)?;

        // allocate first object
        if current.is_some() {
            // allocate new vm object struct on heap
            while let Some(mut object) = current {
                let current_ref = unsafe { object.as_mut() };

                if let Some(mut prev) = current_ref.prev {
                    let prev_ref = unsafe { prev.as_mut() };
                    let new_base = prev_ref.base + prev_ref.length as u64;

                    // allocate between previous object and current one
                    if new_base + (length as u64) < current_ref.base {
                        base = new_base;
                        let new_object = unsafe {
                            VmObject::alloc_new(base, length, flags, current, current_ref.prev)
                        };

                        prev_ref.next = Some(new_object);
                        current_ref.prev = Some(new_object);
                        break;
                    }
                } else {
                    // allocate new object before the first one, if possible
                    if (length as u64) < current_ref.base {
                        base = 0;
                        let new_object =
                            unsafe { VmObject::alloc_new(base, length, flags, current, None) };
                        current_ref.prev = Some(new_object);
                        break;
                    }
                }

                // allocate after last object
                if current_ref.next.is_none() {
                    base = current_ref.base + current_ref.length as u64;
                    let new_object =
                        unsafe { VmObject::alloc_new(base, length, flags, None, current) };
                    current_ref.next = Some(new_object);
                    break;
                }
                // continue with new object
                current = current_ref.next;
            }
        } else {
            let new_object = unsafe { VmObject::alloc_new(base, length, flags, None, None) };
            self.head = Some(new_object);
        }

        // map pages for newly allocated vm object
        let page_count = length / PAGE_SIZE;
        self.pages_allocated += page_count;

        let vmm_start = self.vmm_start;
        let ptm = self.ptm();
        // immediate backing
        for page in 0..page_count {
            let physical_address = match allocation_type {
                AllocationType::AnyPages => ptm
                    .pmm()
                    .request_page()
                    .map_err(|err| VmmError::Paging(err.into()))?,
                AllocationType::Address(address) => address + (page * PAGE_SIZE) as u64,
            };

            let virtual_address = vmm_start + base + (page * PAGE_SIZE) as u64;
            ptm.map_memory(
                virtual_address,
                physical_address,
                PageEntryFlags::from(flags),
            )
            .map_err(|err| VmmError::Paging(err.into()))?;
            // clear newly allocated region
            if !flags.contains(VmFlags::MMIO) && flags.contains(VmFlags::WRITE) {
                unsafe {
                    (virtual_address as *mut u8).write_bytes(0, PAGE_SIZE);
                }
            }
        }

        Ok(self.vmm_start + base)
    }

    /// Frees an allocated VMM-object.
    pub(crate) fn free(&mut self, address: VirtualAddress) -> Result<(), VmmError> {
        if address >= self.vmm_start {
            return Err(VmmError::InvalidRequest(address));
        }

        let mut current = self.head;
        while let Some(current_ref) = current {
            let current_ref = unsafe { current_ref.as_ref() };

            // check for requested object
            if current_ref.base == address - self.vmm_start {
                let ptm = self.ptm();

                let page_count = current_ref.length / PAGE_SIZE;
                // free regions in vmm memory segment
                for page in 0..page_count {
                    // unmap virtual address
                    let physical_address = ptm
                        .mappings()
                        .unmap_memory(address + (page * PAGE_SIZE) as u64)
                        .ok_or(VmmError::InvalidRequest(address))?;

                    // free physical page frames
                    if !current_ref.flags.contains(VmFlags::MMIO) {
                        ptm.pmm()
                            .free_frame(physical_address)
                            .map_err(|err| VmmError::Paging(err.into()))?;
                    }
                }

                self.pages_allocated -= page_count;

                // remove object from linked list
                let heap_ptr = if let Some(mut prev) = current_ref.prev {
                    let prev_ref = unsafe { prev.as_mut() };
                    let heap_ptr = prev_ref.next.unwrap().as_ptr();
                    prev_ref.next = current_ref.next;
                    heap_ptr
                } else {
                    let heap_ptr = self.head.unwrap().as_ptr();
                    self.head = current_ref.next;

                    heap_ptr
                };

                if let Some(mut next) = current_ref.next {
                    let next_ref = unsafe { next.as_mut() };
                    next_ref.prev = current_ref.prev;
                }

                // deallocate vmm struct from heap
                unsafe {
                    dealloc(heap_ptr as *mut u8, Layout::new::<VmObject>());
                }

                return Ok(());
            }

            current = current_ref.next;
        }

        Err(VmmError::InvalidRequest(address))
    }
}

/// Specifies the type of allocation for the virtual memory object
#[derive(Copy, Clone, Debug)]
pub(crate) enum AllocationType {
    AnyPages,
    Address(VirtualAddress),
}
