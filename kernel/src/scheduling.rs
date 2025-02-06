use core::ptr::NonNull;

use alloc::collections::vec_deque::VecDeque;
use mem::{paging::PageTable, VirtualAddress, PAGE_SIZE};
use scheduler::{memory::AddressSpace, task::Process, Scheduler};

use crate::{
    gdt::{KERNEL_CS, KERNEL_DS},
    vmm::{
        error::{PagingError, VmmError},
        object::VmFlags,
        AllocationType, VMM,
    },
};

macro_rules! vmm {
    ($locked:expr) => {{
        $locked
            .get_mut()
            .ok_or(VmmError::VmmUnitialized)
            .map_err(SchedulerError::from)?
    }};
}
struct RoundRobin {
    tasks: VecDeque<Process>,
}

impl Scheduler for RoundRobin {
    const STACK_SIZE: usize = 0x4000;
    const KERNEL_DS: u16 = KERNEL_DS;
    const KERNEL_CS: u16 = KERNEL_CS;

    type SchedulerError = SchedulerError;

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
        mut address_space: AddressSpace,
    ) -> Result<(), Self::SchedulerError> {
        let mut locked = VMM.locked();
        let vmm = vmm!(locked);

        // free all subsequent page tables
        unsafe {
            address_space
                .clean(vmm.ptm().pmm())
                .map_err(PagingError::from)
                .map_err(VmmError::from)
                .map_err(SchedulerError::from)?;

            // free the pml4 frame
            address_space
                .free(vmm.ptm())
                .map_err(PagingError::from)
                .map_err(VmmError::from)
                .map_err(SchedulerError::from)
        }
    }
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

    fn remove_process(&mut self, pid: u64) -> Result<Process, Self::SchedulerError> {
        unimplemented!();
    }
}

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum SchedulerError {
    #[error("{0}")]
    Vmm(#[from] VmmError),
}
