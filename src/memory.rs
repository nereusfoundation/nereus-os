use core::ptr::NonNull;

use bootinfo::BootInfo;
use hal::{PhysicalAddress, PAGE_SIZE};
use uefi::boot::{self, AllocateType, MemoryType};

// custom memory types of the NebulaLoader
pub(crate) const PSF_DATA: MemoryType = MemoryType::custom(0x8000_0000);
pub(crate) const KERNEL_CODE: MemoryType = MemoryType::custom(0x8000_0001);
pub(crate) const KERNEL_STACK: MemoryType = MemoryType::custom(0x8000_0002);
pub(crate) const KERNEL_DATA: MemoryType = MemoryType::custom(0x8000_0003);

pub(crate) const KERNEL_STACK_SIZE: usize = 1024 * 1024; // 1 MB

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

/// Allocate page-sized memory for kernel bootinfo
pub(crate) fn allocate_bootinfo() -> Result<(NonNull<BootInfo>, usize), uefi::Error> {
    let num_pages = size_of::<BootInfo>().div_ceil(PAGE_SIZE);

    boot::allocate_pages(AllocateType::AnyPages, KERNEL_DATA, num_pages)
        .map(|bootinfo| (bootinfo.cast::<BootInfo>(), num_pages))
}
