use bootinfo::BootInfo;
use core::{alloc::GlobalAlloc, cell::OnceCell, ptr};
use mem::{
    error::HeapError, heap::bump::BumpAllocator, paging::PageEntryFlags, KHEAP_PAGE_COUNT,
    KHEAP_VIRTUAL, PAGE_SIZE,
};
use sync::spin::SpinLock;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap(SpinLock::new(OnceCell::new()));

struct LockedHeap(SpinLock<OnceCell<BumpAllocator>>);

pub(crate) fn initialize(bootinfo: &mut BootInfo) -> Result<(), HeapError> {
    let flags = if bootinfo.nx {
        PageEntryFlags::default_nx()
    } else {
        PageEntryFlags::default()
    };

    let ptm = &mut bootinfo.ptm;
    for page in 0..KHEAP_PAGE_COUNT {
        let physical_address = ptm.pmm().request_page()?;

        ptm.map_memory(
            KHEAP_VIRTUAL + (page * PAGE_SIZE) as u64,
            physical_address,
            flags,
        )?;
    }

    let mut lock = ALLOCATOR.0.lock();
    let heap = lock.get_mut_or_init(BumpAllocator::new);
    unsafe {
        heap.init(KHEAP_VIRTUAL, KHEAP_PAGE_COUNT * PAGE_SIZE)?;
    }

    Ok(())
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut bump = self.0.lock();
        bump.get_mut()
            .map(|b| b.alloc(layout).unwrap_or(ptr::null_mut()))
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut bump = self.0.lock();
        if let Some(bump) = bump.get_mut() {
            bump.dealloc(ptr, layout);
        }
    }
}
