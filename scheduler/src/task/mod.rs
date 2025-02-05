use core::ptr::NonNull;

use hal::cpu_state::CpuState;
use mem::paging::ptm::PageTableMappings;

#[derive(Debug)]
pub struct Process {
    pub(crate) stack_top: NonNull<u8>,
    pub(crate) mappings: PageTableMappings,
    pub(crate) pid: u64,
    pub(crate) state: ProcessState,
    pub(crate) context: NonNull<CpuState>,
    pub(crate) next: Option<NonNull<Process>>,
}

impl Process {
    /// Creates a new process instance with no next node and the
    /// `crate::task::ProcessState::Ready`.
    pub fn new(
        stack: NonNull<u8>,
        mappings: PageTableMappings,
        pid: u64,
        context: NonNull<CpuState>,
    ) -> Process {
        Self {
            stack_top: stack,
            mappings,
            pid,
            state: ProcessState::Ready,
            next: None,
            context,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessState {
    Running,
    Ready,
    Done,
    Dead,
}
