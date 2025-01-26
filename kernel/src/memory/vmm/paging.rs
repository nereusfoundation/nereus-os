use bootinfo::BootInfo;
use mem::map::MemoryMap;
use mem::paging::ptm::PageTableManager;
use mem::VirtualAddress;
use mem::{map::MemoryType, PAGE_SIZE, PAS_VIRTUAL, PAS_VIRTUAL_MAX};
use sync::locked::Locked;

use crate::graphics::LOGGER;

use super::error::PagingError;
use super::{AllocationType, VmFlags, VmmError, VMM};

/// Global page table manager, used before the Virtual Memory Manager is set up.
pub(crate) static PTM: Locked<PageTableManager> = Locked::new();

/// Reclaims the memory previously allocated by the bootloader
///
/// This uses the global page table manager and must be called before initializing the virtual
/// memory manager.
pub(crate) fn reclaim_loader_memory(bootinfo: &mut BootInfo) -> Result<(), PagingError> {
    let mmap = bootinfo.mmap;
    let mut locked = PTM.locked();
    let ptm = locked.get_mut().ok_or(PagingError::PtmUnitialized)?;

    let flags = ptm.nx_flags();
    // remap loader
    mmap.descriptors()
        .iter()
        .filter(|desc| desc.r#type == MemoryType::Loader)
        .try_for_each(|desc| {
            (0..desc.num_pages).try_for_each(|page| {
                // unmap from identity mapping
                ptm.mappings()
                    .unmap_memory(desc.phys_start + PAGE_SIZE as u64 * page);

                if desc.phys_end < PAS_VIRTUAL_MAX {
                    // remap to PAS offset
                    ptm.map_memory(
                        desc.phys_start + PAS_VIRTUAL + PAGE_SIZE as u64 * page,
                        desc.phys_start + PAGE_SIZE as u64 * page,
                        flags,
                    )?;
                }
                Ok::<(), PagingError>(())
            })
        })?;

    // unsreserve loader memory
    unsafe { ptm.pmm().use_loader_memory().map_err(PagingError::from) }
}

/// Remaps the framebuffer from the initial identity-mapping to MMIO managed by the VMM.
pub(crate) fn remap_framebuffer() -> Result<(), VmmError> {
    let mut llocked = LOGGER.locked();
    let logger = llocked
        .get_mut()
        .expect("logger must be initialized when remapping framebuffer");

    let mut vlocked = VMM.locked();
    let vmm = vlocked.get_mut().ok_or(VmmError::VmmUnitialized)?;

    let framebuffer = logger.framebuffer().ptr();
    let old_address = framebuffer as *mut u8 as VirtualAddress;
    let page_count = framebuffer.len().div_ceil(PAGE_SIZE);

    // map the framebuffer as MMIO
    let address = vmm.alloc(
        framebuffer.len(),
        VmFlags::WRITE | VmFlags::MMIO,
        AllocationType::Address(old_address),
    )?;

    // unmap the old identity-mapping
    (0..page_count).for_each(|page| {
        vmm.ptm()
            .mappings()
            .unmap_memory(old_address + (page * PAGE_SIZE) as u64)
            .expect("old framebuffer addresses must be mapped");
    });

    unsafe {
        logger.framebuffer().update_ptr(address as *mut u8);
    }

    Ok(())
}

/// Reclaims the memory previously used by the ACPI
///
/// This uses the virtual memory manager and must be called after it's initialization. This
/// function must only be called after the ACPI tables have been parsed.
pub(crate) fn reclaim_acpi_memory(mmap: MemoryMap) -> Result<(), VmmError> {
    let mut locked = VMM.locked();
    let vmm = locked.get_mut().ok_or(VmmError::VmmUnitialized)?;
    let ptm = vmm.ptm();
    let flags = ptm.nx_flags();

    // remap acpi data
    mmap.descriptors()
        .iter()
        .filter(|desc| desc.r#type == MemoryType::AcpiData)
        .try_for_each(|desc| {
            (0..desc.num_pages).try_for_each(|page| {
                // unmap from identity mapping
                ptm.mappings()
                    .unmap_memory(desc.phys_start + PAGE_SIZE as u64 * page);

                if desc.phys_end < PAS_VIRTUAL_MAX {
                    // remap to PAS offset
                    ptm.map_memory(
                        desc.phys_start + PAS_VIRTUAL + PAGE_SIZE as u64 * page,
                        desc.phys_start + PAGE_SIZE as u64 * page,
                        flags,
                    )?;
                }
                Ok::<(), PagingError>(())
            })
        })?;

    // unsreserve acpi data memory
    unsafe {
        ptm.pmm()
            .use_acpi_memory()
            .map_err(|err| VmmError::Paging(err.into()))
    }
}
