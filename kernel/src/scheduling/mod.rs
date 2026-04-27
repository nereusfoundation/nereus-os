use alloc::collections::vec_deque::VecDeque;
use core::{arch::asm, fmt::Debug, ptr::NonNull};
use error::SchedulerError;
use framebuffer::color;
use hal::{cpu_state::CpuState, hlt_loop, interrupts::without_interrupts};
use mem::{
    paging::{ptm::PageTableMappings, PageTable},
    VirtualAddress, PAGE_SIZE,
};
use scheduler::{
    memory::AddressSpace,
    task::{Task, TaskStack, TaskState},
    Scheduler,
};
use sync::locked::Locked;

use crate::{
    gdt::{KERNEL_CS, KERNEL_DS},
    loginfo,
    memory::vmm,
    print, serial_println,
    vmm::{error::VmmError, object::VmFlags, AllocationType, VMM},
};

mod error;

pub(crate) fn initialize() -> Result<(), SchedulerError> {
    SCHEDULER.initialize(PerCoreScheduler::try_new(init, idle)?);
    Ok(())
}

/// Cleanup function called after a task finishes.
fn exit() -> ! {
    // update task state
    without_interrupts(|| {
        let mut locked = SCHEDULER.locked();

        let Some(scheduler) = locked.get_mut() else {
            unreachable!();
        };

        let task = scheduler
            .ready_tasks
            .iter_mut()
            .find(|x| x.pid() == scheduler.active_pid)
            .unwrap();

        task.set_state(TaskState::Done);
    });

    // send interrupt to trigger scheduler
    unsafe {
        asm!("int 32");
    }

    hlt_loop();
}

fn idle() {
    loginfo!("now idle");
    serial_println!("idle");
    hlt_loop();
}

fn test() {
    loginfo!("now test");
    serial_println!("test");
    exit();
}

fn init() {
    loginfo!("now init");
    serial_println!("init");

    // add new process
    {
        let mut sched = SCHEDULER.locked();
        let sched = sched.get_mut().unwrap();
        sched.insert_process(test).unwrap();
    }

    hlt_loop();
}

static SCHEDULER: Locked<PerCoreScheduler> = Locked::new();

macro_rules! vmm {
    ($locked:expr) => {{
        $locked.get_mut().ok_or(VmmError::VmmUnitialized)?
    }};
}
#[derive(Debug)]
pub(crate) struct PerCoreScheduler {
    ready_tasks: VecDeque<Task>,
    active_pid: u64,
    idle_pid: u64,
    pid_counter: u64,
}

impl PerCoreScheduler {
    /// Initializes a new scheduler with an init and an idle task.
    pub(crate) fn try_new(init: fn(), idle: fn()) -> Result<PerCoreScheduler, SchedulerError> {
        let idle = PerCoreScheduler::create_process(0, idle)?;
        let init = PerCoreScheduler::create_process(1, init)?;
        let idle_pid = idle.pid();

        let ready_tasks = VecDeque::from([idle, init]);

        let active_pid = idle_pid;

        let pid_counter = 2;

        Ok(PerCoreScheduler {
            ready_tasks,
            active_pid,
            idle_pid,
            pid_counter,
        })
    }
    fn activate_task(task: &mut Task) {
        task.activate().unwrap();
        // update VMM
        unsafe {
            vmm::update(task.mappings()).unwrap();
        }
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
            .alloc(PAGE_SIZE, VmFlags::WRITE, AllocationType::AnyPages)?
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
            address_space.clean(vmm.ptm().pmm())?;

            // free the pml4 frame
            address_space.free(vmm.ptm()).map_err(SchedulerError::from)
        }
    }

    /// Allocates a new task stack using the global virtual memory manager.
    ///
    /// Note: Memory allocated by the VMM is guaranteeed to be 16-byte-aligned. [`mem::VMM_VIRTUAL`] and subsequent addresses are multiples of 16.
    fn allocate_stack() -> Result<TaskStack, Self::SchedulerError> {
        let mut locked = VMM.locked();
        let vmm = vmm!(locked);

        let btm = vmm
            .alloc(Self::STACK_SIZE, VmFlags::WRITE, AllocationType::AnyPages)
            .map_err(SchedulerError::from)?;
        Ok(TaskStack::new(unsafe { btm.add(Self::STACK_SIZE) }, btm))
    }

    fn free_stack(stack_bottom: NonNull<u8>) -> Result<(), Self::SchedulerError> {
        serial_println!("freeing stack");
        let mut locked = VMM.locked();
        let vmm = vmm!(locked);

        vmm.free(stack_bottom.as_ptr() as VirtualAddress)
            .map_err(SchedulerError::from)
    }

    /// Removes the process from the queue of tasks. This only succeeds if the process has the state
    /// [`scheduler::task::TaskState::Done`].
    fn remove_process(&mut self, task_pid: u64) -> Task {
        let idx = self
            .ready_tasks
            .iter()
            .position(|x| x.pid() == task_pid)
            .expect("process to be removed must exist");

        let task = self.ready_tasks.remove(idx).unwrap();
        assert_eq!(task.state(), TaskState::Done);
        task
    }

    fn add_process(&mut self, process: Task) {
        assert_eq!(process.state(), TaskState::Ready);
        assert_eq!(process.pid(), self.pid_counter - 1);
        self.ready_tasks.push_front(process);
    }

    fn run(context: &CpuState) -> &CpuState {
        let mut scheduler = SCHEDULER.locked();
        let Some(scheduler) = scheduler.get_mut() else {
            return context;
        };
        // first pause the current task
        {
            let current_task_pid = scheduler.active_pid;
            let current_task = scheduler
                .ready_tasks
                .iter_mut()
                .find(|x| x.pid() == current_task_pid)
                .expect("active task must be part of task queue.");

            match current_task.state() {
                TaskState::Ready => {
                    // will be freed
                    let mut pre_mappings = {
                        let mut locked = VMM.locked();
                        let vmm = locked.get_mut().unwrap();
                        let ptm = vmm.ptm().mappings_ref();
                        let pml4 = ptm.pml4_virtual();
                        let pml4_phys = ptm.pml4_physical();

                        AddressSpace::new_standalone(pml4_phys, pml4, ptm.nx())
                    };

                    unsafe {
                        PerCoreScheduler::delete_address_space(&mut pre_mappings);
                    }
                    // first time a task is scheduled
                    PerCoreScheduler::activate_task(current_task);

                    serial_println!("i deleted pre-mt stuff");

                    // free pre-mappings
                    return unsafe { current_task.context().as_ref() };
                }
                TaskState::Done => {
                    serial_println!("current task is done! (id: {})", current_task_pid);
                    assert_ne!(current_task_pid, scheduler.idle_pid);
                    current_task.pause().expect("scheduler paused - failed");

                    // remove finsihed task
                    scheduler.kill_process(current_task_pid).unwrap();
                }
                TaskState::Running => {
                    serial_println!("switch");
                    // set old address space & task to deactived
                    current_task.pause().expect("scheduler paused - failed");
                    current_task.update(context); // update state to current one
                }
            }
        }
        // then activate the next task
        let mut next_task = scheduler
            .ready_tasks
            .pop_back()
            .expect("at least idle task must be present");

        scheduler.active_pid = next_task.pid();
        PerCoreScheduler::activate_task(&mut next_task);
        let context = unsafe { next_task.context().as_ref() };
        scheduler.ready_tasks.push_front(next_task);

        context
    }

    fn insert_process(&mut self, entry: fn()) -> Result<(), Self::SchedulerError> {
        let pid = self.pid_counter;
        self.pid_counter += 1;

        let task = PerCoreScheduler::create_process(pid, entry)?;
        self.add_process(task);
        Ok(())
    }
}
