use core::ptr::NonNull;

use hal::cpu_state::CpuState;
use mem::paging::ptm::PageTableMappings;

use crate::memory::{AddressSpace, AddressSpaceError, State};

#[derive(Debug)]
pub struct Task {
    pub(crate) stack: TaskStack,
    pub(crate) address_space: AddressSpace,
    pub(crate) pid: u64,
    pub(crate) state: TaskState,
    pub(crate) context: NonNull<CpuState>,
}

#[derive(Debug)]
pub struct TaskStack {
    pub(crate) top: NonNull<u8>,
    pub(crate) bottom: NonNull<u8>,
}
impl TaskStack {
    pub fn new(top: NonNull<u8>, bottom: NonNull<u8>) -> Self {
        Self { top, bottom }
    }

    pub fn top(&self) -> NonNull<u8> {
        self.top
    }
    pub fn bottom(&self) -> NonNull<u8> {
        self.bottom
    }

    /// Effectively "allocates" `size` bytes on the stack.
    ///
    /// Note: the "allocation" is force-aligned to 16 bytes.
    pub fn sub_aligned(&mut self, size: usize) {
        let rsp = unsafe { self.top.sub(size) };
        self.top = unsafe { NonNull::new_unchecked(((rsp.as_ptr() as usize) & !0xF) as *mut u8) };
    }
}

impl Task {
    /// Creates a new task instance with the
    /// [`crate::task::TaskState::Ready`] state.
    pub fn new(
        stack: TaskStack,
        address_space: AddressSpace,
        pid: u64,
        context: NonNull<CpuState>,
    ) -> Task {
        Self {
            stack,
            address_space,
            pid,
            state: TaskState::Ready,
            context,
        }
    }
}

impl Task {
    pub fn pid(&self) -> u64 {
        self.pid
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn context(&self) -> NonNull<CpuState> {
        self.context
    }
}
impl Task {
    /// Sets the task state to [`crate::task::TaskState::Ready`] and the address space to
    /// [`crate::memory::State::Inactive`]. If the task is done, the VAS is just changed to inactive. This fails if the VAS is poisoned.
    pub fn pause(&mut self) -> Result<(), TaskError> {
        if self.state == TaskState::Done {
            self.address_space.state = State::Inactive;
            Ok(())
        } else if self.address_space.state == State::Poisoned {
            Err(TaskError::VasPoisoned)
        } else {
            self.state = TaskState::Ready;
            self.address_space.state = State::Inactive;
            Ok(())
        }
    }

    /// Sets the task state to [`crate::task::TaskState::Running`] and the address space to
    /// [`crate::memory::State::Active`]. This fails if the current task state is
    /// [`crate::task::TaskState::Done`] or the VAS is poisoned.
    pub fn activate(&mut self) -> Result<(), TaskError> {
        if self.state == TaskState::Done {
            Err(TaskError::Done)
        } else if self.address_space.state == State::Poisoned {
            Err(TaskError::VasPoisoned)
        } else {
            self.state = TaskState::Running;
            self.address_space.state = State::Active;
            unsafe {
                self.address_space.activate_unchecked();
            }
            Ok(())
        }
    }

    /// Creates a copy of the taks's page table mappings and returns it.
    pub fn mappings(&self) -> PageTableMappings {
        self.address_space.copy_mappings()
    }

    /// Updates the task's context.
    pub fn update(&mut self, new: &CpuState) {
        self.context = NonNull::from_ref(new);
    }

    /// Updates the task's state.
    pub fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskState {
    Running,
    Ready,
    Done,
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Requested operation cannot be performed on a task that has finished already.")]
    Done,
    #[error("The address space of the task is invalid.")]
    VasPoisoned,
    #[error("{0}")]
    Vas(#[from] AddressSpaceError),
}
