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
    /// Offset used to access page tables after enabling new paging scheme. Defaults to 0.
    offset: VirtualAddress,
    /// Whether to map certain pages as non-executable
    nx: bool,
}

impl PageTableMappings {
    pub fn new(pml4: NonNull<PageTable>, nx: bool) -> PageTableMappings {
        PageTableMappings {
            pml4,
            offset: 0,
            nx,
        }
    }
}

impl PageTableMappings {
    pub fn pml4_physical(&self) -> NonNull<PageTable> {
        self.pml4
    }

    pub fn pml4_virtual(&self) -> NonNull<PageTable> {
        unsafe {
            self.pml4
                .cast::<u8>()
                .add(self.offset as usize)
                .cast::<PageTable>()
        }
    }

    pub fn offset(&self) -> VirtualAddress {
        self.offset
    }

    /// Used to make page table manager accessible after enabling direct mapping paging scheme with offset. Updates page table manager to use offset when traversing page tables.
    ///
    /// # Safety
    /// The caller must ensure that the offset is valid.
    pub unsafe fn update_offset(&mut self, offset: VirtualAddress) {
        self.offset = offset;
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

        unsafe { self.invalidate_tlb_entry(physical_address) };

        Some(physical_address)
    }

    /// Used to update cache when unmapping addresses
    ///
    /// # Safety
    ///
    /// The caller has to ensure that the address is the appropriate one and no longer mapped.
    pub unsafe fn invalidate_tlb_entry(&self, virtual_address: VirtualAddress) {
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
