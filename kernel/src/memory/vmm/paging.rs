use bootinfo::BootInfo;
use mem::paging::ptm::PageTableManager;
use mem::{map::MemoryType, PAGE_SIZE, PAS_VIRTUAL, PAS_VIRTUAL_MAX};
use sync::locked::Locked;

use super::error::PagingError;

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
        .filter(|desc| desc.phys_end < PAS_VIRTUAL_MAX && desc.r#type == MemoryType::Loader)
        .try_for_each(|desc| {
            (0..desc.num_pages).try_for_each(|page| {
                // unmap from identity mapping
                ptm.mappings()
                    .unmap_memory(desc.phys_start + PAGE_SIZE as u64 * page);

                // remap to PAS offset
                ptm.map_memory(
                    desc.phys_start + PAS_VIRTUAL + PAGE_SIZE as u64 * page,
                    desc.phys_start + PAGE_SIZE as u64 * page,
                    flags,
                )
            })
        })?;

    // unsreserve loader memory
    unsafe { ptm.pmm().use_loader_memory().map_err(PagingError::from) }
}
