use alloc::{collections::linked_list::LinkedList, rc::Rc};
use core::{cell::RefCell, ptr::NonNull};
use error::SchedulerError;
use hal::{cpu_state::CpuState, hlt_loop};
use mem::{paging::PageTable, VirtualAddress, PAGE_SIZE};
use scheduler::{memory::AddressSpace, task::Task, Scheduler};
use sync::locked::Locked;

use crate::{
    gdt::{KERNEL_CS, KERNEL_DS},
    serial_println,
    vmm::{error::VmmError, object::VmFlags, AllocationType, VMM},
};

mod error;

pub(crate) fn initialize() -> Result<(), SchedulerError> {
    SCHEDULER.initialize(PerCoreScheduler::try_new(idle)?);
    Ok(())
}

fn idle() {
    serial_println!("now idle");
    hlt_loop();
}

fn test() {
    serial_println!("now tst");
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
#[derive(Debug)]
pub(crate) struct PerCoreScheduler {
    tasks: LinkedList<Rc<RefCell<Task>>>,
    active: Rc<RefCell<Task>>,
    idle: Rc<RefCell<Task>>,
    pid_counter: u64,
}

fn dump(t: &Task) {
    let mut locked = VMM.locked();
    let vmm = locked.get_mut().unwrap();
    serial_println!("current: {:?}", unsafe {
        vmm.ptm().mappings_ref().pml4_virtual().as_ref().entries
    });

    serial_println!("task: {:?}", unsafe {
        t.mappings().pml4_virtual().as_ref().entries
    });
}

impl PerCoreScheduler {
    /// Initializes a new scheduler with an idle task.
    pub(crate) fn try_new(_idle: fn()) -> Result<PerCoreScheduler, SchedulerError> {
        todo!("create new per core scheduler")
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

        let pml4_phys = vmm
            .ptm()
            .mappings()
            .get(pml4.as_ptr() as u64)
            .unwrap()
            .cast::<PageTable>();
        Ok(AddressSpace::new(pml4_phys, pml4, vmm.ptm()))
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
    /// [`scheduler::task::TaskState::Done`].
    fn remove_process(&mut self, _pid: u64) -> Result<Rc<RefCell<Task>>, Self::SchedulerError> {
        unimplemented!();
    }

    fn add_process(&mut self, _process: Task) -> Result<(), Self::SchedulerError> {
        unimplemented!();
    }

    fn run(context: &CpuState) -> &CpuState {
        /*
        // set old address space & task to deactived
        current_task.pause().expect("scheduler paused - failed");
        let mut next_task = next_task.borrow_mut();
        next_task.activate().unwrap();
        // update VMM
        unsafe {
            vmm::update(next_task.mappings()).unwrap();
        }
        return unsafe { next_task.context().as_ref() };
        */
        context
    }
}
