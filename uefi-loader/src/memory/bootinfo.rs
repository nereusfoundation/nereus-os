use core::ptr::NonNull;

use alloc::vec::Vec;
use bootinfo::BootInfo;
use mem::{PAGE_SIZE, PAS_VIRTUAL_MAX};
use uefi::{
    boot::{self, AllocateType},
    mem::memory_map::MemoryMap,
};

use super::{NebulaMemoryDescriptor, KERNEL_DATA, MMAP_META_DATA};

// Allocate page-sized memory for kernel bootinfo and set up vector of memory map descriptors
pub(crate) fn allocate_bootinfo(
) -> Result<(NonNull<BootInfo>, Vec<NebulaMemoryDescriptor>), uefi::Error> {
    let num_pages = size_of::<BootInfo>().div_ceil(PAGE_SIZE);
    let ptr = boot::allocate_pages(
        AllocateType::MaxAddress(PAS_VIRTUAL_MAX),
        KERNEL_DATA,
        num_pages,
    )
    .map(|bootinfo| bootinfo.cast::<BootInfo>())?;

    // get uefi memory map meta data to allocate a sufficient number of bytes for the nebula memory map in advance
    let len = boot::memory_map(MMAP_META_DATA)?.meta().map_size;
    let descriptors = allocate_memory_map(len)?;
    Ok((ptr, descriptors))
}

fn allocate_memory_map(cap: usize) -> Result<Vec<NebulaMemoryDescriptor>, uefi::Error> {
    assert_eq!(
        align_of::<Vec<NebulaMemoryDescriptor>>(),
        0x8,
        "invalid memory descriptor alignment"
    );
    let num_pages = cap.div_ceil(PAGE_SIZE);
    // consider changing this because min data addr is subtracted??
    // allocate pages to be able to specify the maximum address
    let ptr = boot::allocate_pages(
        AllocateType::MaxAddress(PAS_VIRTUAL_MAX),
        KERNEL_DATA,
        num_pages,
    )?
    .cast::<NebulaMemoryDescriptor>()
    .as_ptr();

    Ok(unsafe { Vec::from_raw_parts(ptr, 0, cap) })
}
