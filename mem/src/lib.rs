#![no_std]

pub mod map;
pub mod bitmap_allocator;
pub mod error;
pub mod paging;

pub type PhysicalAddress = u64;
pub type VirtualAddress = u64;

pub const PAGE_SIZE: usize = 4096;

/// Size of initial kernel stack
pub const KERNEL_STACK_SIZE: usize = 1024 * 16; // 16 KB

/// Virtual offset of kernel stack (starting at 0)
pub const KERNEL_STACK_VIRTUAL: VirtualAddress = 0xffff_ffff_ffff_ffff - KERNEL_STACK_SIZE as u64;
/// Virtual offset of kernel data (starting at 0): includes acpi(will later be unmapped), bootinfo, psf data
pub const KERNEL_DATA_VIRTUAL: VirtualAddress = 0xffff_ffff_7000_0000;
/// Virtual offset of kernel code (starting at 0)
pub const KERNEL_CODE_VIRTUAL: VirtualAddress = 0xffff_ffff_8000_0000;

