use core::slice;

use crate::{
    PAGE_SIZE, PAS_VIRTUAL_MAX, PhysicalAddress,
    error::FrameAllocatorError,
    map::{MemoryDescriptor, MemoryMap, MemoryType},
};
use map::BitMap;

pub mod map;

#[derive(Debug)]
pub struct BitMapAllocator {
    memory_map: MemoryMap,
    bit_map: BitMap,
    current_descriptor_index: usize,
    current_address: PhysicalAddress,
    free_memory: u64,
    used_memory: u64,
    reserved_memory: u64,
    /// Whether to use Loader and BootService memory types as available or ignore them. Defaults to
    /// true.
    ignore_loader: bool,
    /// Whether to use ACPI Table as available memory or ignore them. Defautlts to true.
    ignore_acpi: bool,
}

impl BitMapAllocator {
    /// Attempts to initialize a new bitmap allocator with the given memory map
    pub fn try_new(memory_map: MemoryMap) -> Result<BitMapAllocator, FrameAllocatorError> {
        // total memory size in bytes => / PAGE_SIZE is the amount of pages. In the bitmap each page is one bit => /8 gives out the amount of bytes to allocate
        let total_pages = (memory_map.last_addr as usize).div_ceil(PAGE_SIZE);
        let bit_map_size = total_pages.div_ceil(8);

        // find memory region to store bitmap in
        let mem = memory_map
            .descriptors()
            .iter()
            .filter(|mem| {
                mem.phys_end < PAS_VIRTUAL_MAX
                    && mem.r#type == MemoryType::Available
                    && mem.size() >= bit_map_size as u64
            })
            .min_by(|a, b| a.size().cmp(&b.size()))
            .ok_or(FrameAllocatorError::InvalidMemoryMap)?;

        let mem_ptr = mem.phys_start as *mut u8;

        let buffer = unsafe { slice::from_raw_parts_mut(mem_ptr, bit_map_size) };

        // clear any pre-existing data
        buffer.fill(0);

        let bit_map = BitMap::new(buffer);

        let free_memory = total_memory(&memory_map);

        let mut instance = Self {
            memory_map,
            bit_map,
            free_memory,
            used_memory: 0,
            reserved_memory: 0,
            current_descriptor_index: 0,
            current_address: 0,
            ignore_loader: true,
            ignore_acpi: true,
        };

        // reserve frames of bitmap
        instance.reserve_frames(mem_ptr as u64, instance.bit_map.pages())?;

        // reserve frames for reserved memory descriptors
        let mmap = instance.memory_map;
        mmap.descriptors()
            .iter()
            .filter(|desc| desc.r#type != MemoryType::Available)
            .try_for_each(|desc| {
                instance.reserve_frames(desc.phys_start, desc.num_pages as usize)
            })?;
        Ok(instance)
    }
}

impl BitMapAllocator {
    /// Returns any available free page
    pub fn request_page(&mut self) -> Result<PhysicalAddress, FrameAllocatorError> {
        for desc_index in self.current_descriptor_index..self.memory_map.descriptors().len() {
            let desc = &self.memory_map.descriptors()[desc_index];

            if desc.r#type == MemoryType::Available
                || !self.ignore_loader && desc.r#type == MemoryType::Loader
                || !self.ignore_acpi && desc.r#type == MemoryType::AcpiData
            {
                for addr in
                    (self.current_address.max(desc.phys_start)..desc.phys_end).step_by(PAGE_SIZE)
                {
                    let index = addr / PAGE_SIZE as u64;
                    if !self.bit_map.get(index)? {
                        self.allocate_frame(addr)?;
                        self.current_descriptor_index = desc_index;
                        self.current_address = addr + PAGE_SIZE as u64;
                        return Ok(addr);
                    }
                }
            }
            self.current_address = desc.phys_start;
        }

        // if no free page is found, start from the beginning
        self.current_descriptor_index = 0;
        self.current_address = 0;

        // todo: page frame swap
        Err(FrameAllocatorError::NoMoreFreePages)
    }
}

impl BitMapAllocator {
    /// Attempt to allocate a single free frame
    pub fn allocate_frame(&mut self, address: PhysicalAddress) -> Result<(), FrameAllocatorError> {
        let index = address / PAGE_SIZE as u64;
        if self.bit_map.get(index)? {
            return Err(FrameAllocatorError::OperationFailed(address));
        }

        self.bit_map.set(index, true)?;
        self.free_memory -= PAGE_SIZE as u64;
        self.used_memory += PAGE_SIZE as u64;

        Ok(())
    }

    /// Attempt to allocate a series of free frames
    pub fn allocate_frames(
        &mut self,
        start_address: PhysicalAddress,
        page_count: usize,
    ) -> Result<(), FrameAllocatorError> {
        for i in 0..page_count {
            self.allocate_frame(start_address + (i * PAGE_SIZE) as u64)?;
        }

        Ok(())
    }

    /// Attempt to free a single allocated frame
    pub fn free_frame(&mut self, address: PhysicalAddress) -> Result<(), FrameAllocatorError> {
        let index = address / PAGE_SIZE as u64;
        if !self.bit_map.get(index)? {
            return Err(FrameAllocatorError::OperationFailed(address));
        }

        self.bit_map.set(index, false)?;
        self.free_memory += PAGE_SIZE as u64;
        self.used_memory -= PAGE_SIZE as u64;

        Ok(())
    }

    /// Attempt to free a series of allocated frames
    pub fn free_frames(
        &mut self,
        start_address: PhysicalAddress,
        page_count: usize,
    ) -> Result<(), FrameAllocatorError> {
        for i in 0..page_count {
            self.free_frame(start_address + (i * PAGE_SIZE) as u64)?;
        }

        Ok(())
    }

    /// Attempt to reserve a single free frame
    pub fn reserve_frame(&mut self, address: PhysicalAddress) -> Result<(), FrameAllocatorError> {
        let index = address / PAGE_SIZE as u64;
        if self.bit_map.get(index)? {
            return Err(FrameAllocatorError::OperationFailed(address));
        }

        self.bit_map.set(index, true)?;
        self.free_memory -= PAGE_SIZE as u64;
        self.reserved_memory += PAGE_SIZE as u64;

        Ok(())
    }

    /// Attempt to reserve a series of free frames
    pub fn reserve_frames(
        &mut self,
        start_address: PhysicalAddress,
        page_count: usize,
    ) -> Result<(), FrameAllocatorError> {
        for i in 0..page_count {
            self.reserve_frame(start_address + (i * PAGE_SIZE) as u64)?;
        }

        Ok(())
    }

    /// Attempt to reserve a single free frame
    pub fn free_reserved_frame(
        &mut self,
        address: PhysicalAddress,
    ) -> Result<(), FrameAllocatorError> {
        let index = address / PAGE_SIZE as u64;
        if !self.bit_map.get(index)? {
            return Err(FrameAllocatorError::OperationFailed(address));
        }

        self.bit_map.set(index, false)?;
        self.free_memory += PAGE_SIZE as u64;
        self.reserved_memory -= PAGE_SIZE as u64;

        Ok(())
    }

    /// Attempt to free a series of reserved frames
    pub fn free_reserved_frames(
        &mut self,
        start_address: PhysicalAddress,
        page_count: usize,
    ) -> Result<(), FrameAllocatorError> {
        for i in 0..page_count {
            self.free_reserved_frame(start_address + (i * PAGE_SIZE) as u64)?;
        }

        Ok(())
    }
}

impl BitMapAllocator {
    // Returns the amount of free memory in bytes
    pub fn free_memory(&self) -> u64 {
        self.free_memory
    }
    /// Returns the amount of used memory in bytes
    pub fn used_memory(&self) -> u64 {
        self.used_memory
    }

    /// Returns the amount of reserved memory in bytes
    pub fn reserved_memory(&self) -> u64 {
        self.reserved_memory
    }
}

impl BitMapAllocator {
    /// Update the bitmap buffer pointer. Mainly used to make the allocator available after
    /// switching to a new paging scheme
    ///
    /// # Safety
    /// Caller must guarantee that the new offset pointer is valid.
    pub unsafe fn update_bit_map_ptr(&mut self, offset: u64) {
        unsafe {
            let old = self.bit_map.ptr() as u64;
            // todo: handle case of buffer overflow
            self.bit_map.update_ptr((offset + old) as *mut u8);
        }
    }

    /// Update the memory map descriptor pointer. Mainly used to make the allocator avaiable after
    /// switching to a new paging scheme
    ///
    /// # Safety
    /// Caller must guarantee that the new offset pointer is valid.
    pub unsafe fn update_memory_map_ptr(&mut self, offset: u64) {
        let old = self.memory_map.descriptors;
        // todo: handle case of buffer overflow
        self.memory_map.descriptors = (offset + old as u64) as *mut MemoryDescriptor;
    }

    /// Make the Loader and BootService memory types available.
    ///
    /// # Safety
    /// Caller must ensure that this function can be called. Must only be called from the kernel.
    pub unsafe fn use_loader_memory(&mut self) -> Result<(), FrameAllocatorError> {
        self.ignore_loader = false;
        let mmap = self.memory_map;
        mmap.descriptors()
            .iter()
            .filter(|desc| desc.r#type == MemoryType::Loader)
            .try_for_each(|desc| {
                self.free_reserved_frames(desc.phys_start, desc.num_pages as usize)
            })
    }

    /// Make the ACPI Tables memory types available.
    ///
    /// # Safety
    /// Caller must ensure that this function can be called.
    pub unsafe fn use_acpi_memory(&mut self) -> Result<(), FrameAllocatorError> {
        self.ignore_acpi = false;
        let mmap = self.memory_map;
        mmap.descriptors()
            .iter()
            .filter(|desc| desc.r#type == MemoryType::AcpiData)
            .try_for_each(|desc| {
                self.free_reserved_frames(desc.phys_start, desc.num_pages as usize)
            })
    }
}

impl BitMapAllocator {
    pub fn address(&mut self) -> u64 {
        unsafe { self.bit_map.ptr() as u64 }
    }

    pub fn pages(&mut self) -> usize {
        self.bit_map.pages()
    }
}
/// Returns total amount of memory in bytes based on memory map.
pub fn total_memory(mmap: &MemoryMap) -> u64 {
    mmap.descriptors().iter().map(|desc| desc.size()).sum()
}
