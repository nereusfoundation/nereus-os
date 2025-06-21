use crate::PhysicalAddress;

#[derive(Debug, thiserror::Error)]
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
#[derive(Debug, thiserror::Error)]
pub enum HeapError {
    #[error("Out of memory")]
    Oom,
    #[error("Allocation out of bounds")]
    Oob,
    #[error("Invalid heap block size {0}")]
    InvalidBlockSize(usize),
}
