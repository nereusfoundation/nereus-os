use core::slice;

use alloc::vec::Vec;
use goblin::elf::program_header::PT_LOAD;
use mem::{PhysicalAddress, VirtualAddress, PAGE_SIZE};
use uefi::boot;

use crate::{error::ElfParseError, memory::KERNEL_CODE};

/// Executable Linkable Format wrapper for parsing elfs
#[derive(Copy, Clone, Debug)]
pub(crate) struct Elf {
    /// Virutal entry point of the elf
    entry_point: VirtualAddress,
    /// Physical file base address
    ///
    /// > since uefi sets up identity paging this address can be used directly
    file_base: PhysicalAddress,
    /// Number of pages of the elf
    num_pages: usize,
}

impl Elf {
    /// Retrieve entry point address
    pub(crate) fn entry(&self) -> VirtualAddress {
        self.entry_point
    }

    /// Retrieve file base address
    pub(crate) fn base(&self) -> PhysicalAddress {
        self.file_base
    }

    /// Retrieve number of pages
    pub(crate) fn num_pages(&self) -> usize {
        self.num_pages
    }
}

impl Elf {
    /// Parse elf and allocate memory for it
    pub(crate) fn try_new(data: Vec<u8>) -> Result<Elf, ElfParseError> {
        let data = data.as_slice();
        let elf = goblin::elf::Elf::parse(data).map_err(ElfParseError::from)?;
        let mut dest_start = u64::MAX;
        let mut dest_end = 0;

        if !elf.is_64 {
            return Err(ElfParseError::InvalidFormat);
        }

        // set up range of memory needed to be allocated
        for pheader in elf.program_headers.iter() {
            // skip non-load segments (e.g.: dynamic linking info)
            if pheader.p_type != PT_LOAD {
                continue;
            }

            dest_start = dest_start.min(pheader.p_paddr);
            dest_end = dest_end.max(pheader.p_paddr + pheader.p_memsz);
        }
        let num_pages = (dest_end as usize - dest_start as usize).div_ceil(PAGE_SIZE);

        // allocate file data
        assert_eq!(
            dest_start,
            boot::allocate_pages(
                boot::AllocateType::Address(dest_start),
                KERNEL_CODE,
                num_pages,
            )
            .map_err(ElfParseError::from)?
            .as_ptr() as u64
        );

        // Copy program segments of elf into memory
        for pheader in elf.program_headers.iter() {
            // skip non-load segments (e.g.: dynamic linking info)
            if pheader.p_type != PT_LOAD {
                continue;
            }
            let base_address = pheader.p_paddr;
            let offset = pheader.p_offset as usize;
            let size_in_file = pheader.p_filesz as usize;
            let size_in_memory = pheader.p_memsz as usize;

            let dest =
                unsafe { slice::from_raw_parts_mut(base_address as *mut u8, size_in_memory) };
            dest[..size_in_file].copy_from_slice(&data[offset..offset + size_in_file]);
            dest[size_in_file..].fill(0);
        }

        Ok(Elf {
            entry_point: elf.entry,
            file_base: dest_start,
            num_pages,
        })
    }
}
