#![no_std]
#![feature(pointer_is_aligned_to)]

use core::ptr::NonNull;

use hal::{cpu_state::CpuState, registers::rflags::RFlags};
use memory::AddressSpace;
use task::Task;

use crate::task::TaskStack;
pub mod memory;
pub mod task;

extern crate alloc;
// note: for now a process is a mix of process and thread, it will be extended later on.

pub trait Scheduler {
    type SchedulerError;

    /// Size of the task's stack.
    ///
    /// Note: the stack is used in the beginning to store the initial task [`hal::cpu_state::CpuState`]. Thus, the available size is smaller.
    const STACK_SIZE: usize;
    const KERNEL_DS: u16;
    const KERNEL_CS: u16;

    /// Allocates the stack for a new thread. Returning the address of the stack top and the stack bottom (in that order).
    fn allocate_stack() -> Result<TaskStack, Self::SchedulerError>;

    /// Frees the stack starting at the specified address.
    fn free_stack(stack_bottom: NonNull<u8>) -> Result<(), Self::SchedulerError>;

    /// Creates a new virtual address space for a new process. Returning the new
    /// mappings.
    fn create_address_space() -> Result<AddressSpace, Self::SchedulerError>;

    /// Deletes the specified virtual address space.
    ///
    /// # Safety
    /// The page mappings of the process are not automatically invalidated. This is only an issue
    /// if the currently active address space is manipulated.
    /// [`memory::AddressSpace::clean()`] for more information.
    unsafe fn delete_address_space(
        address_space: &mut AddressSpace,
    ) -> Result<(), Self::SchedulerError>;

    /// Removes the previously active done process from the queue of tasks.
    fn remove_process(&mut self, task_pid: u64) -> Task;
    /// Adds a process to the queue of tasks.
    fn add_process(&mut self, process: Task);

    /// Creates a new process.
    ///
    /// Note: the process is not automatically added to any queues. See [`Scheduler::insert_process()`] as an alternative.
    fn create_process(pid: u64, entry: fn()) -> Result<Task, Self::SchedulerError> {
        let mut stack = Self::allocate_stack()?;

        // put inital cpu sate onto stack
        stack.sub_aligned(size_of::<CpuState>());
        assert!(stack.top.is_aligned_to(16));

        let context = stack.top.cast::<CpuState>();

        let mappings = Self::create_address_space()?;

        let flags = RFlags::RESERVED_1 | RFlags::INTERRUPTS_ENABLED;

        unsafe {
            context.write(CpuState::new(
                Self::KERNEL_DS.into(),
                stack.top.as_ptr() as u64,
                flags,
                Self::KERNEL_CS.into(),
                entry as usize as u64,
                0, // indicates we have reached the top-most stack frame
            ));
        }

        Ok(Task::new(stack, mappings, pid, context))
    }

    /// Creates and inserts a new process into the queue of ready tasks.
    fn insert_process(&mut self, entry: fn()) -> Result<(), Self::SchedulerError>;

    /// Deletes the previously active done process, cleaning up it's memory and removing it from the queue of
    /// tasks.
    fn kill_process(&mut self, pid: u64) -> Result<(), Self::SchedulerError> {
        // remove process from queue
        let mut process = self.remove_process(pid);

        // free stack
        Self::free_stack(process.stack.bottom)?;

        unimplemented!("need to switch to global mappings");
        // free mappings
        unsafe {
            Self::delete_address_space(&mut process.address_space)?;
        }

        Ok(())
    }

    /// Schedules the next task.
    fn run(context: &CpuState) -> &CpuState;
}
