use core::ptr::NonNull;

use mem::{
    bitmap_allocator::BitMapAllocator,
    error::FrameAllocatorError,
    paging::{
        ptm::{PageTableManager, PageTableMappings},
        PageEntryFlags, PageTable,
    }, VirtualAddress,
};

/// Owns page memory mappings and keeps track of the process' allocated frames. The higher-half of
/// the virtual address space is shared accross all processes.
#[derive(Debug)]
pub struct AddressSpace {
    mappings: PageTableMappings,
    state: State
}

impl AddressSpace {
    /// Creates a new private address space. The caller must have allocated a frame for the pml4
    /// page table in advance. The higher-half mappings, as well as the nx-feature are copied from the page table manager. The address space is [`State::Inactive`] per default.
    pub fn new(pml4: NonNull<PageTable>, ptm: &PageTableManager) -> AddressSpace {
        let mut mappings = PageTableMappings::new(pml4, ptm.nx());

        // copy higher half
        ptm.mappings_ref().copy(&mut mappings);
        AddressSpace{mappings, state: State::Inactive}
    }

    /// Frees the level 4 page table and unmaps it.
    pub fn free(&mut self, ptm: &mut PageTableManager) -> Result<(), AddressSpaceError> {
        match self.state {
            State::Inactive => { // unmap pml4 in current mapping. 

        // SAFETY: this must NOT be called if the specified address space is active. (todo: add an
        // active flag to VAS)
        let address = self.mappings.pml4_virtual().as_ptr() as VirtualAddress;
        ptm.mappings().unmap_memory(address).ok_or(FrameAllocatorError::OperationFailed(address))?;

        // free frame
        ptm.pmm().free_frame(address).map_err(AddressSpaceError::from)
}
State::Active => Err(AddressSpaceError::FreeActive),
State::Poisoned => Err(AddressSpaceError::FreePoisoned)
        }
        
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
            return Err(AddressSpaceError::CleanPoisoned);
        }

        let pml4 = unsafe { self.mappings.pml4_virtual().as_mut() };
        let offset = self.mappings.offset();
        // iterate over each pml4 entry for the lower half
        pml4.entries[0..256]
            .iter_mut()
            .filter(|entry| entry.flags().contains(PageEntryFlags::PRESENT))
            .try_for_each(|entry| -> Result<(), FrameAllocatorError> {
                let level3 = unsafe {
                    ((entry.address() + offset) as *mut PageTable)
                        .as_mut()
                        .ok_or(FrameAllocatorError::OperationFailed(entry.address()))?
                };

                // iterate over each level 3 entry
                level3
                    .entries
                    .iter_mut()
                    .filter(|entry| entry.flags().contains(PageEntryFlags::PRESENT))
                    .try_for_each(|entry| -> Result<(), FrameAllocatorError> {
                        let level2 = unsafe {
                            ((entry.address() + offset) as *mut PageTable)
                                .as_mut()
                                .ok_or(FrameAllocatorError::OperationFailed(entry.address()))?
                        };

                        // iterate over each level 2 entry
                        level2
                            .entries
                            .iter_mut()
                            .filter(|entry| entry.flags().contains(PageEntryFlags::PRESENT))
                            .try_for_each(|entry|  
                                // free level 1 table frame
                                pmm.free_frame(entry.address())
                            )?;

                        // free level 2 table frame
                        pmm.free_frame(entry.address())
                    })?;

                // free level 3 table frame
                pmm.free_frame(entry.address())?;

                // reset page entry
                entry.set_address(0);
                entry.set_flags(PageEntryFlags::empty());
                
                // SAFETY: the mapping is NOT invalidated here!

               Ok(())

            }).map_err(AddressSpaceError::from)
    }
}

#[derive(Debug, thiserror_no_std::Error)]
pub enum AddressSpaceError {
    #[error("{0}")]
    FrameAllocator(#[from] FrameAllocatorError),
    #[error("Cannot free poisoned address space.")]
    FreePoisoned,
    #[error("Cannot free active address space.")]
    FreeActive,
    #[error("Cannot clean poisoned address space.")]
    CleanPoisoned
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Active,
    Inactive, 
    /// If the address space has been freed.
    Poisoned
}
