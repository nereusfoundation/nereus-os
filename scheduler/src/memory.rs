use core::ptr::NonNull;

use mem::{
    VirtualAddress,
    bitmap_allocator::BitMapAllocator,
    error::FrameAllocatorError,
    paging::{
        PageTable,
        ptm::{PageTableManager, PageTableMappings},
    },
};

/// Owns page memory mappings and keeps track of the process' allocated frames. The higher-half of
/// the virtual address space is shared accross all processes.
#[derive(Debug)]
pub struct AddressSpace {
    mappings: PageTableMappings,
    pub(crate) state: State,
}

impl AddressSpace {
    /// Creates a new private address space. The caller must have allocated a frame for the pml4
    /// page table in advance. The higher-half mappings, as well as the nx-feature and offset are copied from the page table manager. The address space is [`State::Inactive`] per default.
    pub fn new(
        pml4_phys: NonNull<PageTable>,
        pml4_virt: NonNull<PageTable>,
        ptm: &PageTableManager,
    ) -> AddressSpace {
        let mut mappings = PageTableMappings::new(pml4_phys, ptm.nx());

        // copy offset
        unsafe {
            mappings.update_offset(ptm.mappings_ref().offset());
        }

        // set pml4 virtual address
        unsafe {
            mappings.update_pml4_virtual(pml4_virt);
        }

        // copy higher half
        ptm.mappings_ref().copy(&mut mappings);

        AddressSpace {
            mappings,
            state: State::Inactive,
        }
    }

    /// Activates the address space without checking whether it's poisoned.
    ///
    /// # Safety
    /// This can destroy the virtual memory if the VAS is invalid.
    pub unsafe fn activate_unchecked(&mut self) {
        self.state = State::Active;
        let cr3 = self.mappings.pml4_physical();
        let addr = cr3.as_ptr() as u64;
        unsafe {
            core::arch::asm!("mov cr3, {}", in(reg) addr);
        };
    }

    /// Frees the level 4 page table and unmaps it.
    pub fn free(&mut self, ptm: &mut PageTableManager) -> Result<(), AddressSpaceError> {
        match self.state {
            State::Inactive => {
                // unmap pml4 in current mapping.

                // SAFETY: this must NOT be called if the specified address space is active. (todo: add an
                // active flag to VAS)
                let address = self.mappings.pml4_virtual().as_ptr() as VirtualAddress;
                ptm.mappings()
                    .unmap_memory(address)
                    .ok_or(FrameAllocatorError::OperationFailed(address))?;

                // free frame
                ptm.pmm()
                    .free_frame(address)
                    .map_err(AddressSpaceError::from)
            }
            State::Active => Err(AddressSpaceError::FreeActive),
            State::Poisoned => Err(AddressSpaceError::FreePoisoned),
        }
    }

    /// Creates a copy of the VAS' memory mappings.
    pub fn copy_mappings(&self) -> PageTableMappings {
        let mut cpy = PageTableMappings::new(self.mappings.pml4_physical(), self.mappings.nx());
        unsafe {
            cpy.update_offset(self.mappings.offset());
        }
        unsafe {
            cpy.update_pml4_virtual(self.mappings.pml4_virtual());
        }
        cpy
    }

    pub fn take_mappings(self) -> PageTableMappings {
        self.mappings
    }

    /// Frees the lower-half page tables of the mappings.
    ///
    /// Note: The PML4 and higher half entries are
    /// still valid after this operation.
    ///
    /// # Safety
    /// The pages previously mapped to the lower half are no longer accessible after this action.
    /// The caller must invalidate these entries manually or switch to a new paging scheme.
    pub unsafe fn clean(&mut self, pmm: &mut BitMapAllocator) -> Result<(), AddressSpaceError> {
        if self.state == State::Poisoned {
            Err(AddressSpaceError::CleanPoisoned)
        } else {
            // SAFETY: the mapping is NOT invalidated here!
            unsafe { self.mappings.clean(pmm, false) }.map_err(AddressSpaceError::from)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddressSpaceError {
    #[error("{0}")]
    FrameAllocator(#[from] FrameAllocatorError),
    #[error("Cannot free poisoned address space.")]
    FreePoisoned,
    #[error("Cannot free active address space.")]
    FreeActive,
    #[error("Cannot clean poisoned address space.")]
    CleanPoisoned,
    #[error("Cannot activate a poisoned address space.")]
    ActivatePosioned,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Active,
    Inactive,
    /// If the address space has been freed.
    Poisoned,
}
