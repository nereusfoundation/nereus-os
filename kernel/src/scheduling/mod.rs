use alloc::rc::Rc;
use core::{cell::RefCell, ptr::NonNull};
use hal::{cpu_state::CpuState, hlt_loop};
use mem::{paging::PageTable, VirtualAddress, PAGE_SIZE};
use queue::TaskQueue;
use scheduler::{
    memory::{AddressSpace, AddressSpaceError},
    task::Process,
    Scheduler,
};
use sync::locked::Locked;

use crate::{
    gdt::{KERNEL_CS, KERNEL_DS},
    vmm::{error::VmmError, object::VmFlags, AllocationType, VMM},
};

mod queue;

pub(crate) fn initialize() -> Result<(), SchedulerError> {
    SCHEDULER.initialize(PerCoreScheduler::try_new(idle)?);
    Ok(())
}

fn idle() {
    hlt_loop();
}

static SCHEDULER: Locked<PerCoreScheduler> = Locked::new();

macro_rules! vmm {
    ($locked:expr) => {{
        $locked
            .get_mut()
            .ok_or(VmmError::VmmUnitialized)
            .map_err(SchedulerError::from)?
    }};
}
pub(crate) struct PerCoreScheduler {
    tasks: TaskQueue,
    active: Rc<RefCell<Process>>,
    idle: Rc<RefCell<Process>>,
    pid_counter: u64,
}
impl PerCoreScheduler {
    /// Initializes a new scheduler with an idle task.
    pub(crate) fn try_new(idle: fn()) -> Result<PerCoreScheduler, SchedulerError> {
        let task = <PerCoreScheduler as Scheduler>::create_process(0, idle)?;
        let task = Rc::new(RefCell::new(task));
        let idle = Rc::clone(&task);
        let active = Rc::clone(&idle);
        Ok(Self {
            idle,
            active,
            tasks: TaskQueue,
            pid_counter: 1,
        })
    }
}

impl Scheduler for PerCoreScheduler {
    const STACK_SIZE: usize = 0x4000;
    const KERNEL_DS: u16 = KERNEL_DS;
    const KERNEL_CS: u16 = KERNEL_CS;

    type SchedulerError = SchedulerError;

    /// Creates a new address space for a task using the global virtual memory maanger.
    ///
    /// Note: Memory allocated by the VMM is guaranteed to be page-aligned. [`mem::VMM_VIRTUAL`] and subsequent addresses are multiples of [`mem::PAGE_SIZE`].
    fn create_address_space() -> Result<AddressSpace, Self::SchedulerError> {
        let mut locked = VMM.locked();
        let vmm = vmm!(locked);

        let pml4 = vmm
            .alloc(PAGE_SIZE, VmFlags::WRITE, AllocationType::AnyPages)
            .map_err(SchedulerError::from)?
            .cast::<PageTable>();

        Ok(AddressSpace::new(pml4, vmm.ptm()))
    }

    unsafe fn delete_address_space(
        address_space: &mut AddressSpace,
    ) -> Result<(), Self::SchedulerError> {
        let mut locked = VMM.locked();
        let vmm = vmm!(locked);

        // free all subsequent page tables
        unsafe {
            address_space
                .clean(vmm.ptm().pmm())
                .map_err(SchedulerError::from)?;

            // free the pml4 frame
            address_space.free(vmm.ptm()).map_err(SchedulerError::from)
        }
    }

    /// Allocates a new task stack using the global virtual memory manager.
    ///
    /// Note: Memory allocated by the VMM is guaranteeed to be 16-byte-aligned. [`mem::VMM_VIRTUAL`] and subsequent addresses are multiples of 16.
    fn allocate_stack() -> Result<NonNull<u8>, Self::SchedulerError> {
        let mut locked = VMM.locked();
        let vmm = vmm!(locked);

        vmm.alloc(Self::STACK_SIZE, VmFlags::WRITE, AllocationType::AnyPages)
            .map_err(SchedulerError::from)
    }
    fn free_stack(stack_top: NonNull<u8>) -> Result<(), Self::SchedulerError> {
        let mut locked = VMM.locked();
        let vmm = vmm!(locked);

        vmm.free(stack_top.as_ptr() as VirtualAddress)
            .map_err(SchedulerError::from)
    }

    /// Removes a process from the queue of tasks. This only succeeds if the process has the state
    /// [`scheduler::task::ProcessState::Done`].
    fn remove_process(&mut self, _pid: u64) -> Result<Rc<RefCell<Process>>, Self::SchedulerError> {
        unimplemented!();
    }

    fn add_process(&mut self, _process: Process) -> Result<(), Self::SchedulerError> {
        unimplemented!();
    }

    fn schedule(context: NonNull<CpuState>) -> NonNull<CpuState> {
        context
    }
}

#[derive(Debug, thiserror_no_std::Error)]
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
