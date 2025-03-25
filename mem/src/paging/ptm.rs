use core::{arch::asm, ptr::NonNull};

use crate::{
    bitmap_allocator::BitMapAllocator, error::FrameAllocatorError, PhysicalAddress, VirtualAddress,
};

use super::{index::PageMapIndexer, PageEntryFlags, PageTable};

/// Manages Page Table Mappings
#[derive(Debug)]
pub struct PageTableManager {
    frame_allocator: BitMapAllocator,
    mappings: PageTableMappings,
}

impl PageTableManager {
    /// Create a new page table manager instance. By default, a virtual `offset` of 0 is used. This can be changed manually using [`PageTableMappings::update_offset()`].
    pub fn new(
        pml4: NonNull<PageTable>,
        frame_allocator: BitMapAllocator,
        nx: bool,
    ) -> PageTableManager {
        PageTableManager {
            mappings: PageTableMappings::new(pml4, nx),
            frame_allocator,
        }
    }
}

impl PageTableManager {
    /// Get a mutable reference of the physical page frame allocator.
    pub fn pmm(&mut self) -> &mut BitMapAllocator {
        &mut self.frame_allocator
    }

    /// Get a mutable reference to the page table mappings.
    pub fn mappings(&mut self) -> &mut PageTableMappings {
        &mut self.mappings
    }

    /// Get an immutable reference to the page table mappings.
    pub fn mappings_ref(&self) -> &PageTableMappings {
        &self.mappings
    }
    /// Get a mutable refernce to the physical frame allocator and page table mappings.
    pub fn inner(&mut self) -> (&mut PageTableMappings, &mut BitMapAllocator) {
        (&mut self.mappings, &mut self.frame_allocator)
    }

    /// Attempts to get NX-PageEntryFlags if NX is configured.
    pub fn nx_flags(&self) -> PageEntryFlags {
        if self.nx() {
            PageEntryFlags::default_nx()
        } else {
            PageEntryFlags::default()
        }
    }

    /// Whether the NX-feature is enabled.
    pub fn nx(&self) -> bool {
        self.mappings.nx
    }

    /// Used to switch to a new page mappings scheme.
    ///
    /// # Safety
    /// The caller must ensure that the mappings are valid.
    pub unsafe fn update_mappings(&mut self, mappings: PageTableMappings) {
        self.mappings = mappings;
    }

 }

impl PageTableManager {
    /// Map the given virtual address to the physical address
    pub fn map_memory(
        &mut self,
        virtual_address: VirtualAddress,
        physical_address: PhysicalAddress,
        flags: PageEntryFlags,
    ) -> Result<(), FrameAllocatorError> {
        let (mappings, pmm) = self.inner();
        mappings.map_memory(virtual_address, physical_address, flags, pmm)
    }
}

/// Mutable collection of page table entries
#[derive(Debug)]
pub struct PageTableMappings {
    /// Virtual address of level 4 page table
    pml4: NonNull<PageTable>,
    /// Offset used to access page tables after enabling new paging scheme. Defaults to 0. This
    /// offset is not used to refer to the pml4. It has it's own offset.
    offset: VirtualAddress,
    /// Address used to access the virtual address of the pml4. Defaults to physical address of
    /// pml4..
    pml4_virtual: NonNull<PageTable>,
    /// Whether to map certain pages as non-executable
    nx: bool,
}

impl PageTableMappings {
    pub fn new(pml4: NonNull<PageTable>, nx: bool) -> PageTableMappings {
        PageTableMappings {
            pml4,
            offset: 0,
            pml4_virtual: pml4,
            nx,
        }
    }
}

impl PageTableMappings {
    pub fn pml4_physical(&self) -> NonNull<PageTable> {
        self.pml4
    }

    /// Address used to make the pml4 available after enabling a new paging scheme.
    pub fn pml4_virtual(&self) -> NonNull<PageTable> {
        self.pml4_virtual
    }

    /// Offset used to access page tables after enabling new paging scheme. This
    /// offset is not used to refer to the pml4. It has it's own offset which can be retrieved
    /// using [`PageTableMappings::pml4_offset()`].
    pub fn offset(&self) -> VirtualAddress {
        self.offset
    }

    pub fn nx(&self) -> bool {
        self.nx
    }

    /// Used to make page table manager accessible after enabling direct mapping paging scheme with offset. Updates page table manager to use offset when traversing subsequent page tables.
    ///
    /// Note: This offset is not used for the pml4. See [`PageTableMappings::update_pml4_virtual()`]
    ///
    /// # Safety
    /// The caller must ensure that the offset is valid.
    pub unsafe fn update_offset(&mut self, offset: VirtualAddress) {
        self.offset = offset;
    }

    /// Used to make page table manager accessible after enabling direct mppaing paging scheme with
    /// offset. Updates virtual address of the pml4.
    ///
    /// # Safety
    /// The caller must ensure that the address is valid.
    pub unsafe fn update_pml4_virtual(&mut self, pml4_virtual: NonNull<PageTable>) {
        self.pml4_virtual = pml4_virtual;
    }

       /// Frees the lower-half page tables of the mappings. 
    ///
    /// Note: The PML4 and higher half entries are
    /// still valid after this operation. Furthermore, invalidating after cleaning does not just
    /// invalidate the pages that were cleaned, but rather flushes the entire TLB. 
    ///
    /// # Safety
    /// The pages previously mapped to the lower half are no longer accessible after this action.
    /// Furthermore, if "invalidated" the mapping is also activated.
    pub unsafe fn clean(&mut self, pmm: &mut BitMapAllocator, invalidate: bool) -> Result<(), FrameAllocatorError> {

        let pml4 = unsafe { self.pml4_virtual().as_mut() };
        let offset = self.offset();
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
                
                if invalidate {
                    // flush the entire tlb by reloading the c3 register
                    unsafe { core::arch::asm!("mov cr3, {}", in(reg) self.pml4_physical().as_ptr() as usize); }
                }

               Ok(())

            })
    }

}

impl PageTableMappings {
    /// Map the given virtual address to the physical address
    pub fn map_memory(
        &mut self,
        virtual_address: VirtualAddress,
        physical_address: PhysicalAddress,
        flags: PageEntryFlags,
        pmm: &mut BitMapAllocator,
    ) -> Result<(), FrameAllocatorError> {
        let indexer = PageMapIndexer::new(virtual_address);
        let pml4 = self.pml4_virtual();
        let user = flags.contains(PageEntryFlags::USER_SUPER);

        // Map Level 3
        let page_map_level3 = self.get_or_create_next_table(pml4, indexer.pdp_i(), pmm, user)?;
        // Map Level 2
        let page_map_level2 =
            self.get_or_create_next_table(page_map_level3, indexer.pd_i(), pmm, user)?;
        // Map Level 1
        let mut page_map_level1 =
            self.get_or_create_next_table(page_map_level2, indexer.pt_i(), pmm, user)?;

        let page_entry = &mut unsafe { page_map_level1.as_mut() }.entries[indexer.p_i() as usize];

        page_entry.set_address(physical_address);
        page_entry.set_flags(flags);

        Ok(())
    }

    /// Remove the mapping for given virtual address. Returns the physical address the virtual address previously pointed to.
    pub fn unmap_memory(&mut self, virtual_memory: VirtualAddress) -> Option<PhysicalAddress> {
        let indexer = PageMapIndexer::new(virtual_memory);
        let page_map_level4 = self.pml4_virtual();
        // Map Level 3
        let page_map_level3 = self.get_next_table(page_map_level4, indexer.pdp_i())?;
        // Map Level 2
        let page_map_level2 = self.get_next_table(page_map_level3, indexer.pd_i())?;
        // Map Level 1
        let mut page_map_level1 = self.get_next_table(page_map_level2, indexer.pt_i())?;

        let page_entry = &mut unsafe { page_map_level1.as_mut() }.entries[indexer.p_i() as usize];
        let physical_address = page_entry.address();

        page_entry.set_address(0);
        page_entry.set_flags(PageEntryFlags::empty());

        unsafe { Self::invalidate_tlb_entry(physical_address) };

        Some(physical_address)
    }

    /// Retrieves the physical address of the provided virtual address
    pub fn get(&self, virtual_memory: VirtualAddress) -> Option<NonNull<u8>> {
        let indexer = PageMapIndexer::new(virtual_memory);
        let page_map_level4 = self.pml4_virtual();
        // Map Level 3
        let page_map_level3 = self.get_next_table(page_map_level4, indexer.pdp_i())?;
        // Map Level 2
        let page_map_level2 = self.get_next_table(page_map_level3, indexer.pd_i())?;
        // Map Level 1
        let mut page_map_level1 = self.get_next_table(page_map_level2, indexer.pt_i())?;

        let page_entry = &mut unsafe { page_map_level1.as_mut() }.entries[indexer.p_i() as usize];
        let physical_address = page_entry.address();

        Some(unsafe { NonNull::new_unchecked(physical_address as *mut u8) })
    }

    /// Used to update cache when unmapping addresses
    ///
    /// # Safety
    ///
    /// The caller has to ensure that the address is the appropriate one and no longer mapped.
    pub unsafe fn invalidate_tlb_entry(virtual_address: VirtualAddress) {
        unsafe {
            asm!("invlpg [{}]", in(reg) virtual_address as *const u8);
        }
    }

    /// Copies the higher-half page tables from the current mappings to the destination instance.
    /// The higher-half of the address space is shared between processes. (more info: <https://www.kernel.org/doc/html/v5.8/x86/x86_64/mm.html>)
    pub fn copy(&self, other: &mut PageTableMappings) {
        unsafe {
            other.pml4_virtual().as_mut().entries[256..]
                .copy_from_slice(&self.pml4_virtual().as_ref().entries[256..]);
        }
    }

    /// Attempt the get the next table
    fn get_next_table(
        &self,
        mut current_table: NonNull<PageTable>,
        index: u64,
    ) -> Option<NonNull<PageTable>> {
        let entry = &mut unsafe { current_table.as_mut() }.entries[index as usize];
        if entry.flags().contains(PageEntryFlags::PRESENT) {
            unsafe {
                Some(NonNull::new_unchecked(
                    (entry.address() + self.offset) as *mut PageTable,
                ))
            }
        } else {
            None
        }
    }

    /// Get a pointer to next table or create it if it does not exist yet.
    fn get_or_create_next_table(
        &mut self,
        mut current_table: NonNull<PageTable>,
        index: u64,
        pmm: &mut BitMapAllocator,
        user: bool,
    ) -> Result<NonNull<PageTable>, FrameAllocatorError> {
        let entry = &mut unsafe { current_table.as_mut() }.entries[index as usize];

        if entry.flags().contains(PageEntryFlags::PRESENT) {
            // path to entry user accessible as well
            if user && !entry.flags().contains(PageEntryFlags::USER_SUPER) {
                entry.set_flags(entry.flags() | PageEntryFlags::USER_SUPER);
            }

            Ok(
                unsafe {
                    NonNull::new_unchecked((entry.address() + self.offset) as *mut PageTable)
                },
            )
        } else {
            let new_page = pmm.request_page()?;
            let new_table = (new_page + self.offset) as *mut PageTable;
            unsafe {
                // Zero out the new table
                core::ptr::write_bytes(new_table, 0, 1);
            }

            entry.set_address(new_page);
            entry.set_flags(
                PageEntryFlags::PRESENT
                    | PageEntryFlags::READ_WRITE
                    | if user {
                        PageEntryFlags::USER_SUPER
                    } else {
                        PageEntryFlags::empty()
                    },
            );

            Ok(unsafe { NonNull::new_unchecked(new_table) })
        }
    }
}
