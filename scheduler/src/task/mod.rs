use core::ptr::NonNull;

use hal::cpu_state::CpuState;

use crate::memory::AddressSpace;

#[derive(Debug)]
pub struct Process {
    pub(crate) stack_top: NonNull<u8>,
    pub(crate) address_space: AddressSpace,
    pub(crate) pid: u64,
    pub(crate) state: ProcessState,
    pub(crate) context: NonNull<CpuState>,
}

impl Process {
    /// Creates a new process instance with the
    /// [`crate::task::ProcessState::Ready`] state.
    pub fn new(
        stack: NonNull<u8>,
        address_space: AddressSpace,
        pid: u64,
        context: NonNull<CpuState>,
    ) -> Process {
        Self {
            stack_top: stack,
            address_space,
            pid,
            state: ProcessState::Ready,
            context,
        }
    }
}

impl Process {
    pub fn pid(&self) -> u64 {
        self.pid
    }

    pub fn state(&self) -> ProcessState {
        self.state
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessState {
    Running,
    Ready,
    Done,
}
