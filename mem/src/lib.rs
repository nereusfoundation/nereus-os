#![no_std]

pub mod bitmap_allocator;
pub mod error;
pub mod map;
pub mod paging;

pub type PhysicalAddress = u64;
pub type VirtualAddress = u64;

pub const PAGE_SIZE: usize = 4096;

/// Size of initial kernel stack
pub const KERNEL_STACK_SIZE: usize = 1024 * 16; // 16 KB

/// Virtual offset of kernel stack (mapping starting at 0)
pub const KERNEL_STACK_VIRTUAL: VirtualAddress = 0xffff_ffff_ffff_ffff - KERNEL_STACK_SIZE as u64;
/// Virtual offset of kernel data (mapping starting at 0): includes acpi(will later be unmapped), bootinfo, psf data
pub const KERNEL_DATA_VIRTUAL: VirtualAddress = 0xffff_ffff_b000_0000;
/// Virtual offset of kernel code (mapping starting at 0)
pub const KERNEL_CODE_VIRTUAL: VirtualAddress = 0xffff_ffff_8000_0000;
/// Virtual offset of physical available address space (page table mappings, ...) (directy offset
/// mapping)
pub const PAS_VIRTUAL: VirtualAddress = 0xffff_8000_0000_0000;
/// How many pages of the physical address space to map to the virtual offset
pub const PAS_VIRTUAL_MAX: VirtualAddress = KERNEL_CODE_VIRTUAL - PAS_VIRTUAL;
