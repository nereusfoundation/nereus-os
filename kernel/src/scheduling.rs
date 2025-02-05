use core::ptr::NonNull;

use alloc::collections::vec_deque::VecDeque;
use scheduler::{task::Process, Scheduler};

use crate::{
    gdt::{KERNEL_CS, KERNEL_DS},
    vmm::{error::VmmError, VMM},
};

struct RoundRobin {
    tasks: VecDeque<Process>,
}

impl Scheduler for RoundRobin {
    const STACK_SIZE: u64 = 0x4000;
    const KERNEL_DS: u16 = KERNEL_DS;
    const KERNEL_CS: u16 = KERNEL_CS;

    type SchedulerError = SchedulerError;

    fn create_address_space() -> Result<mem::paging::ptm::PageTableMappings, Self::SchedulerError> {
        let mut locked = VMM.locked();
        let _vmm = locked
            .get_mut()
            .ok_or(VmmError::VmmUnitialized)
            .map_err(SchedulerError::from)?;
        unimplemented!();
    }

    fn delete_address_space(
        mappings: mem::paging::ptm::PageTableMappings,
    ) -> Result<(), Self::SchedulerError> {
        unimplemented!();
    }
    fn allocate_stack() -> Result<NonNull<u8>, Self::SchedulerError> {
        unimplemented!();
    }
    fn free_stack(stack_top: NonNull<u8>) -> Result<(), Self::SchedulerError> {
        unimplemented!();
    }
    fn create_process(&mut self, pid: u64, entry: fn()) -> Result<Process, Self::SchedulerError> {
        unimplemented!();
    }
    fn remove_process(&mut self, pid: u64) -> Result<Process, Self::SchedulerError> {
        unimplemented!();
    }
    fn kill_process(&mut self, pid: u64) -> Result<(), Self::SchedulerError> {
        unimplemented!();
    }
}

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum SchedulerError {
    #[error("{0}")]
    Vmm(#[from] VmmError),
}
