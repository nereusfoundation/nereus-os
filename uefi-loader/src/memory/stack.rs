use mem::{align_down, align_up, PhysicalAddress, PAGE_SIZE};
use uefi::boot::{self, AllocateType};

use super::KERNEL_STACK;

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

impl KernelStack {
    /// Creates a new kernel stack, adhering to the 16-byte alignment rule.
    pub(crate) fn new(
        top: PhysicalAddress,
        bottom: PhysicalAddress,
        num_pages: usize,
    ) -> KernelStack {
        Self {
            top: align_down(top, 0x10),
            bottom: align_up(bottom, 0x10),
            num_pages,
        }
    }
}

/// Allocate kernel stack with the given size in bytes (aligned to upward page-size)
pub(crate) fn allocate_kernel_stack(bytes: usize) -> Result<KernelStack, uefi::Error> {
    let num_pages = bytes.div_ceil(PAGE_SIZE);
    let bottom = boot::allocate_pages(AllocateType::AnyPages, KERNEL_STACK, num_pages)?.as_ptr()
        as PhysicalAddress;
    let top = bottom + (PAGE_SIZE * num_pages) as u64;

    Ok(KernelStack::new(top, bottom, num_pages))
}
