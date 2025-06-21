use scheduler::memory::AddressSpaceError;

use crate::vmm::error::VmmError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum SchedulerError {
    #[error("{0}")]
    Vmm(#[from] VmmError),
    #[error("{0}")]
    AddressSpace(#[from] AddressSpaceError),
    #[error("Process not found: PID{0}")]
    ProcessNotFound(u64),
    #[error("Process with the same PID{0} is already in the queue.")]
    DuplicatePid(u64),
    #[error("Must not remove active task.")]
    RemoveNoDone,
}
