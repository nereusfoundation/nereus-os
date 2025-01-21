use mem::{error::FrameAllocatorError, VirtualAddress};

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum VmmError {
    #[error("Paging error: {0}")]
    Paging(#[from] PagingError),
    #[error("Requested object has not been allocated")]
    InvalidRequest(VirtualAddress),
    #[error("Out of memory")]
    Oom,
    #[error("Virtual Memory Manager has not been intialized")]
    VmmUnitialized,
}

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum PagingError {
    #[error("Frame Allocator Error: {0}")]
    FrameAllocator(#[from] FrameAllocatorError),
    #[error("Page Table Manager has not been intialized")]
    PtmUnitialized,
}
