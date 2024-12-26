use core::ptr::NonNull;

use alloc::vec::Vec;
use bootinfo::{BootInfo, PhysicalAddress, PAGE_SIZE};
use uefi::{
    boot::{self, AllocateType, MemoryType},
    mem::memory_map::MemoryMap,
};

// custom memory types of the NebulaLoader
pub(crate) const PSF_DATA: MemoryType = MemoryType::custom(0x8000_0000);
pub(crate) const KERNEL_CODE: MemoryType = MemoryType::custom(0x8000_0001);
pub(crate) const KERNEL_STACK: MemoryType = MemoryType::custom(0x8000_0002);
pub(crate) const KERNEL_DATA: MemoryType = MemoryType::custom(0x8000_0003);
pub(crate) const MMAP_META_DATA: MemoryType = MemoryType::custom(0x8000_0004);

pub(crate) const KERNEL_STACK_SIZE: usize = 1024 * 1024; // 1 MB

pub(crate) type NebulaMemoryMap = bootinfo::MemoryMap;
pub(crate) type NebulaMemoryDescriptor = bootinfo::MemoryDescriptor;
pub(crate) type NebulaMemoryType = bootinfo::MemoryType;

#[derive(Copy, Clone, Debug)]
pub(crate) struct KernelStack {
    /// Starting address of memory allocated for stack
    ///
    /// > since uefi sets up identity-mapped paging, the virtual and physical addresses are equivalent
    bottom: PhysicalAddress,
    /// Address of stack top
    ///
    /// > since uefi sets up identity-mapped paging, the virtual and physical addresses are equivalent
    top: PhysicalAddress,
    /// Number of stack pages
    num_pages: usize,
}

impl KernelStack {
    pub(crate) fn bottom(&self) -> PhysicalAddress {
        self.bottom
    }
    pub(crate) fn top(&self) -> PhysicalAddress {
        self.top
    }
    pub(crate) fn num_pages(&self) -> usize {
        self.num_pages
    }
}

/// Allocate kernel stack with the given size in bytes (aligned to upward page-size)
pub(crate) fn allocate_kernel_stack(bytes: usize) -> Result<KernelStack, uefi::Error> {
    let num_pages = bytes.div_ceil(PAGE_SIZE);
    let bottom = boot::allocate_pages(AllocateType::AnyPages, KERNEL_STACK, num_pages)?.as_ptr()
        as PhysicalAddress;
    let top = bottom + (PAGE_SIZE * num_pages) as u64;

    Ok(KernelStack {
        bottom,
        top,
        num_pages,
    })
}

/// Allocate page-sized memory for kernel bootinfo and set up vector of memory map descriptors
pub(crate) fn allocate_bootinfo(
) -> Result<(NonNull<BootInfo>, Vec<NebulaMemoryDescriptor>), uefi::Error> {
    let num_pages = size_of::<BootInfo>().div_ceil(PAGE_SIZE);

    let ptr = boot::allocate_pages(AllocateType::AnyPages, KERNEL_DATA, num_pages)
        .map(|bootinfo| bootinfo.cast::<BootInfo>())?;

    // get uefi memory map meta data to allocate a sufficient number of bytes for the nebula memory map in advance
    let size = boot::memory_map(MMAP_META_DATA)?.meta().map_size;

    let descriptors = Vec::with_capacity(size);

    Ok((ptr, descriptors))
}
