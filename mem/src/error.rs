use crate::PhysicalAddress;

#[derive(Debug, thiserror_no_std::Error)]
pub enum FrameAllocatorError {
    #[error("Invalid index to bitmap")]
    InvalidBitMapIndex,
    #[error("Invalid memory map")]
    InvalidMemoryMap,
    #[error("No more free pages available")]
    NoMoreFreePages,
    #[error("Operation failed - frame with the address {0} already allocated/reserved or free")]
    OperationFailed(PhysicalAddress),
}

#[cfg(feature = "alloc")]
#[derive(thiserror_no_std::Error)]
pub enum HeapError {
    #[error("Frame allocation failed: {0}")]
    AllocationFailed(#[from] FrameAllocatorError),
    #[error("Out of memory")]
    Oom,
    #[error("Allocation out of bounds")]
    Oob,
}
