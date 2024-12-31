use crate::PhysicalAddress;
use core::slice;

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct MemoryMap {
    /// Pointer to memory map descriptors
    pub descriptors: *mut MemoryDescriptor,
    /// Length of memory that descriptors occupy in bytes
    pub descriptors_len: u64,
    /// First address of physical address space
    pub first_addr: PhysicalAddress,
    /// First available address of physical address space
    pub first_available_addr: PhysicalAddress,
    /// Last address of physical address space
    pub last_addr: PhysicalAddress,
    /// Last available address of physical address space
    pub last_available_addr: PhysicalAddress,
}

impl MemoryMap {
    pub fn descriptors(&self) -> &[MemoryDescriptor] {
        unsafe { slice::from_raw_parts(self.descriptors, self.descriptors_len as usize) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryDescriptor {
    pub phys_start: PhysicalAddress,
    pub phys_end: PhysicalAddress,
    pub num_pages: u64,
    pub r#type: MemoryType,
}

impl MemoryDescriptor {
    /// Size of memory of descriptor in bytes
    pub fn size(&self) -> u64 {
        self.phys_end - self.phys_start
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum MemoryType {
    Available = 0,
    Reserved = 1,
    /// kernel code file
    KernelCode = 2,
    /// kernel stack
    KernelStack = 3,
    /// boot info, memory map, font data
    KernelData = 4,
    /// acpi tables
    AcpiData = 5,
    /// loader code,data 
    Loader = 6,
}
