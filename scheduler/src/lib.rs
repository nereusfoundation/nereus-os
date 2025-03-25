#![no_std]

use core::{cell::RefCell, ptr::NonNull};

use alloc::rc::Rc;
use hal::{cpu_state::CpuState, registers::rflags::RFlags};
use memory::AddressSpace;
use task::Task;
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

    /// Allocates the stack for a new thread. Returning the address of the stack top.
    fn allocate_stack() -> Result<NonNull<u8>, Self::SchedulerError>;

    /// Frees the stack starting at the specified address.
    fn free_stack(stack_top: NonNull<u8>) -> Result<(), Self::SchedulerError>;

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

    /// Removes a process from the queue of tasks.
    fn remove_process(&mut self, pid: u64) -> Result<Rc<RefCell<Task>>, Self::SchedulerError>;

    /// Adds a process to the queue of tasks.
    fn add_process(&mut self, process: Task) -> Result<(), Self::SchedulerError>;

    /// Creates a new process.
    ///
    /// Note: the process is not automatically added to any queues.
    fn create_process(pid: u64, entry: fn()) -> Result<Task, Self::SchedulerError> {
        let mappings = Self::create_address_space()?;

        let flags = RFlags::RESERVED_1;

        let stack_top = Self::allocate_stack()?;

        // put inital cpu sate onto stack
        let stack = unsafe { stack_top.sub(size_of::<CpuState>()) };

        let context = stack.cast::<CpuState>();

        unsafe {
            context.write(CpuState::new(
                Self::KERNEL_DS.into(),
                stack.as_ptr() as u64,
                flags,
                Self::KERNEL_CS.into(),
                entry as usize as u64,
                0, // indicates we have reached the top-most stack frame
            ));
        }

        Ok(Task::new(stack, mappings, pid, context))
    }

    /// Deletes an existing process, cleaning up it's memory and removing it from the queue of
    /// tasks.
    fn kill_process(&mut self, pid: u64) -> Result<(), Self::SchedulerError> {
        // remove process from queue
        let process = self.remove_process(pid)?;

        // free stack
        Self::free_stack(process.borrow().stack_top)?;

        // free mappings
        unsafe {
            Self::delete_address_space(&mut process.borrow_mut().address_space)?;
        }

        Ok(())
    }

    /// Schedules the next task.
    fn run(context: &CpuState) -> &CpuState;
}
