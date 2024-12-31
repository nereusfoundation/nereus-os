use bootinfo::BootInfo;
use mem::{
    error::FrameAllocatorError, map::MemoryType, paging::PageEntryFlags, PAGE_SIZE, PAS_VIRTUAL,
    PAS_VIRTUAL_MAX,
};

/// Reclaim the memory previously allocated by the bootloader
pub(super) fn reclaim_loader_memory(bootinfo: &mut BootInfo) -> Result<(), FrameAllocatorError> {
    let mmap = bootinfo.mmap;

    // remap loader
    mmap.descriptors()
        .iter()
        .filter(|desc| desc.phys_end < PAS_VIRTUAL_MAX && desc.r#type == MemoryType::Loader)
        .try_for_each(|desc| {
            (0..desc.num_pages).try_for_each(|page| {
                // unmap from identity mapping
                bootinfo
                    .ptm
                    .mappings()
                    .unmap_memory(desc.phys_start + PAGE_SIZE as u64 * page);

                // remap to PAS offset
                bootinfo.ptm.map_memory(
                    desc.phys_start + PAS_VIRTUAL + PAGE_SIZE as u64 * page,
                    desc.phys_start + PAGE_SIZE as u64 * page,
                    PageEntryFlags::default_nx(),
                )
            })
        })?;

    // unsreserve loader memory
    unsafe { bootinfo.ptm.pmm().use_loader_memory() }
}
