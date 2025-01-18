use bootinfo::BootInfo;
use core::{alloc::GlobalAlloc, ptr};
use mem::{
    error::HeapError, heap::bump::BumpAllocator, paging::PageEntryFlags, KHEAP_PAGE_COUNT,
    KHEAP_VIRTUAL, PAGE_SIZE,
};
use sync::locked::Locked;

use super::vmm::paging::{PagingError, PTM};

#[global_allocator]
static ALLOCATOR: HeapWrapper = HeapWrapper(Locked::new());

pub(crate) fn initialize(bootinfo: &mut BootInfo) -> Result<(), HeapErrorExt> {
    let flags = if bootinfo.nx {
        PageEntryFlags::default_nx()
    } else {
        PageEntryFlags::default()
    };
    let mut lock = PTM.locked();
    let ptm = lock
        .get_mut()
        .ok_or(HeapErrorExt::Paging(PagingError::PtmUnitialized))?;

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
    let heap = lock.get_mut_or_init(BumpAllocator::new);
    unsafe {
        heap.init(KHEAP_VIRTUAL, KHEAP_PAGE_COUNT * PAGE_SIZE)?;
    }

    Ok(())
}

struct HeapWrapper(Locked<BumpAllocator>);

unsafe impl GlobalAlloc for HeapWrapper {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut bump = self.0.locked();
        bump.get_mut()
            .map(|b| b.alloc(layout).unwrap_or(ptr::null_mut()))
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut bump = self.0.locked();
        if let Some(bump) = bump.get_mut() {
            bump.dealloc(ptr, layout);
        }
    }
}

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum HeapErrorExt {
    #[error("{0}")]
    Heap(#[from] HeapError),
    #[error("{0}")]
    Paging(#[from] PagingError),
}
