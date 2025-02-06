use core::ptr::NonNull;

use mem::{
    bitmap_allocator::BitMapAllocator,
    error::FrameAllocatorError,
    paging::{
        ptm::{PageTableManager, PageTableMappings},
        PageEntryFlags, PageTable,
    },
};

/// Owns page memory mappings and keeps track of the process' allocated frames. The higher-half of
/// the virtual address space is shared accross all processes.
#[derive(Debug)]
pub struct AddressSpace(PageTableMappings);

impl AddressSpace {
    /// Creates a new private address space. The caller must have allocated a frame for the pml4
    /// page table in advance. The higher-half mappings, as well as the nx-feature are copied from the page table manager.
    pub fn new(pml4: NonNull<PageTable>, ptm: &PageTableManager) -> AddressSpace {
        let mut mappings = PageTableMappings::new(pml4, ptm.nx());

        // copy higher half
        ptm.mappings_ref().copy(&mut mappings);
        AddressSpace(mappings)
    }

    /// Frees the lower-half page tables of the mappings.
    ///
    /// # Safety
    /// The pages previously mapped to the lower half are no longer accessible after this action.
    /// The caller must invalidate these entries manually or switch to a new paging scheme. 
    pub unsafe fn clean(&mut self, pmm: &mut BitMapAllocator) -> Result<(), FrameAllocatorError> {
        let pml4 = unsafe { self.0.pml4_virtual().as_mut() };
        let offset = self.0.offset();
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

            })
    }
}
