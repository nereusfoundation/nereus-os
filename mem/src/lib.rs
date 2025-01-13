#![no_std]

pub mod bitmap_allocator;
pub mod error;
pub mod heap;
pub mod map;
pub mod paging;

pub type PhysicalAddress = u64;
pub type VirtualAddress = u64;

pub const PAGE_SIZE: usize = 0x1000;

/// Size of initial kernel stack
pub const KERNEL_STACK_SIZE: usize = 1024 * 16 * 4; // 64 KB

/// Virtual offset of kernel stack (mapping starting at 0)
pub const KERNEL_STACK_VIRTUAL: VirtualAddress = 0xffff_ffff_ffff_ffff - KERNEL_STACK_SIZE as u64;
/// Virtual offset of kernel code
pub const KERNEL_CODE_VIRTUAL: VirtualAddress = 0xffff_ffff_8000_0000;
/// Virtual offset of physical available address space (page table mappings, ...) (directy offset
/// mapping), kernel data is also mapped here.
pub const PAS_VIRTUAL: VirtualAddress = 0xffff_8000_0000_0000;
/// Highest pbysical address to be able to be mapped into the higher half
pub const PAS_VIRTUAL_MAX: VirtualAddress = KERNEL_CODE_VIRTUAL - PAS_VIRTUAL;
/// Virtual start address of the kernel heap
pub const KHEAP_VIRTUAL: VirtualAddress = 0xffff_ffff_c000_0000;
/// Number of pages max. used by the kernel heap
pub const KHEAP_PAGE_COUNT: usize = 0x100;
