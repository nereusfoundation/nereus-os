use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{self, NonNull},
};
use mem::{
    align_up,
    error::HeapError,
    heap::linked_list::{LinkedListAllocator, ListNode},
    KHEAP_PAGE_COUNT, KHEAP_VIRTUAL, PAGE_SIZE,
};
use sync::locked::Locked;

use super::vmm::{error::PagingError, paging::PTM, VMM};

#[global_allocator]
static ALLOCATOR: HeapWrapper = HeapWrapper(Locked::new());

pub(crate) fn initialize() -> Result<(), HeapErrorExt> {
    let mut lock = PTM.locked();
    let ptm = lock
        .get_mut()
        .ok_or(HeapErrorExt::Paging(PagingError::PtmUnitialized))?;

    let flags = ptm.nx_flags();

    for page in 0..KHEAP_PAGE_COUNT {
        let physical_address = ptm
            .pmm()
            .request_page()
            .map_err(|err| HeapErrorExt::Paging(PagingError::FrameAllocator(err)))?;

        ptm.map_memory(
            KHEAP_VIRTUAL + (page * PAGE_SIZE) as u64,
            physical_address,
            flags,
        )
        .map_err(|err| HeapErrorExt::Paging(PagingError::FrameAllocator(err)))?;
    }

    let mut lock = ALLOCATOR.0.locked();
    let instance =
        unsafe { LinkedListAllocator::try_new(KHEAP_VIRTUAL, KHEAP_PAGE_COUNT * PAGE_SIZE)? };
    lock.get_mut_or_init(|| instance);

    Ok(())
}

struct HeapWrapper(Locked<LinkedListAllocator>);

unsafe impl GlobalAlloc for HeapWrapper {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut heap = ALLOCATOR.0.locked();
        let size = align_up(layout.size() as u64, layout.align()) as usize;
        if let Some(heap) = heap.get_mut() {
            if let Ok(fit_node) = heap.find_fit(size) {
                if heap.split_block(fit_node, size).is_ok() {
                    return fit_node.as_ptr().add(1) as *mut u8;
                }
            } else {
                // check for PTM
                let mut ptm = PTM.locked();
                if let Some(ptm) = ptm.get_mut() {
                    // expand heap
                    if heap.expand(size, ptm).is_ok() {
                        if let Ok(fit_node) = heap.find_fit(size) {
                            if heap.split_block(fit_node, size).is_ok() {
                                return fit_node.as_ptr().add(1) as *mut u8;
                            }
                        }
                    }
                }
                // check for VMM instead
                else {
                    drop(ptm);
                    let mut vmm = VMM.locked();
                    if let Some(vmm) = vmm.get_mut() {
                        if heap.expand(size, vmm.ptm()).is_ok() {
                            if let Ok(fit_node) = heap.find_fit(size) {
                                if heap.split_block(fit_node, size).is_ok() {
                                    return fit_node.as_ptr().add(1) as *mut u8;
                                }
                            }
                        }
                    }
                }
            }
        }
        ptr::null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }
        let mut hlock = self.0.locked();
        if let Some(heap) = hlock.get_mut() {
            let node_ptr = unsafe { (ptr as *mut ListNode).sub(1) };

            let mut node = unsafe { NonNull::new_unchecked(node_ptr) };
            let node_ref = unsafe { node.as_mut() };
            node_ref.free();
            unsafe {
                heap.merge_blocks(node);
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum HeapErrorExt {
    #[error("{0}")]
    Heap(#[from] HeapError),
    #[error("{0}")]
    Paging(#[from] PagingError),
}
