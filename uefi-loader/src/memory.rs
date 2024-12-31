use core::{
    arch::asm,
    ptr::{self, NonNull},
};

use alloc::vec::Vec;
use bootinfo::BootInfo;
use hal::msr::{efer::Efer, ModelSpecificRegister};
use mem::{
    bitmap_allocator::BitMapAllocator,
    error::FrameAllocatorError,
    map,
    paging::{ptm::PageTableManager, PageEntryFlags, PageTable},
    PhysicalAddress, KERNEL_CODE_VIRTUAL, KERNEL_DATA_VIRTUAL, KERNEL_STACK_SIZE,
    KERNEL_STACK_VIRTUAL, PAGE_SIZE, PAS_VIRTUAL, PAS_VIRTUAL_MAX,
};
use uefi::{
    boot::{self, AllocateType, MemoryType},
    mem::memory_map::MemoryMap,
};

use crate::graphics;

// custom memory types of the NebulaLoader
pub(crate) const PSF_DATA: MemoryType = MemoryType::custom(0x8000_0000);
pub(crate) const KERNEL_CODE: MemoryType = MemoryType::custom(0x8000_0001);
pub(crate) const KERNEL_STACK: MemoryType = MemoryType::custom(0x8000_0002);
pub(crate) const KERNEL_DATA: MemoryType = MemoryType::custom(0x8000_0003);
pub(crate) const MMAP_META_DATA: MemoryType = MemoryType::custom(0x8000_0004);

pub(crate) type NebulaMemoryMap = map::MemoryMap;
pub(crate) type NebulaMemoryDescriptor = map::MemoryDescriptor;
pub(crate) type NebulaMemoryType = map::MemoryType;

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

    let ptr = boot::allocate_pool(KERNEL_DATA, cap)?.as_ptr() as *mut NebulaMemoryDescriptor;
    Ok(unsafe { Vec::from_raw_parts(ptr, 0, cap) })
}

/// Wrapper for attributes of the virtual address space for the kernel
#[derive(Debug)]
pub(crate) struct VirtualAddressSpace {
    pub(crate) bootinfo: *mut BootInfo,
    pub(crate) manager: PageTableManager,
    pub(crate) stack: KernelStack,
}

/// Set up higher-half kernel address space
pub(crate) fn initialize_address_space(
    bootinfo: *mut BootInfo,
    mut pmm: BitMapAllocator,
    old_stack: KernelStack,
    fb_base: u64,
    fb_page_count: usize,
) -> Result<(VirtualAddressSpace, bool), FrameAllocatorError> {
    let mut nx = false;
    assert_ne!(bootinfo, ptr::null_mut());
    let bootinfo_ref = unsafe { bootinfo.as_mut().expect("bootinfo ptr must be valid") };

    let memory_map = bootinfo_ref.mmap;

    let pml4_addr = pmm.request_page()?;
    assert_eq!(
        (pml4_addr as usize) % align_of::<PageTable>(),
        0,
        "pml4 pointer is not aligned"
    );

    let pml4 = pml4_addr as *mut PageTable;

    // zero out new table
    unsafe { ptr::write_bytes(pml4, 0, 1) };

    let mut manager = PageTableManager::new(pml4, pmm);

    let first_addr = |mem_types: &[NebulaMemoryType], mem_map: NebulaMemoryMap| {
        mem_map
            .descriptors()
            .iter()
            .filter(|desc| mem_types.contains(&desc.r#type))
            .map(|desc| desc.phys_start)
            .min()
            .ok_or(FrameAllocatorError::InvalidMemoryMap)
    };

    let first_stack_addr = first_addr(&[NebulaMemoryType::KernelStack], memory_map)?;
    let first_data_addr = first_addr(
        &[NebulaMemoryType::KernelData, NebulaMemoryType::AcpiData],
        memory_map,
    )?;

    // map kernel physical address space to canonical higher half (canonical lower half is reserved
    // for userspace)
    memory_map
        .descriptors()
        .iter()
        .try_for_each(|desc| -> Result<(), FrameAllocatorError> {
            let (virtual_base, physical_base, flags) = match desc.r#type {
                // map part of physical address space to higher half
                NebulaMemoryType::Available => {
                    if desc.phys_end < PAS_VIRTUAL_MAX {
                        (PAS_VIRTUAL, desc.phys_start, PageEntryFlags::default_nx())
                    } else {
                        return Ok(());
                    }
                }
                // do not map reserved memory
                NebulaMemoryType::Reserved => return Ok(()),
                NebulaMemoryType::KernelStack => (
                    KERNEL_STACK_VIRTUAL,
                    desc.phys_start - first_stack_addr,
                    PageEntryFlags::default_nx(),
                ),
                NebulaMemoryType::KernelData | NebulaMemoryType::AcpiData => (
                    KERNEL_DATA_VIRTUAL,
                    desc.phys_start - first_data_addr,
                    PageEntryFlags::default_nx(),
                ),
                NebulaMemoryType::KernelCode => (
                    KERNEL_CODE_VIRTUAL,
                    desc.phys_start,
                    PageEntryFlags::default(),
                ),
                // loader data, code pages will later be reclaimed by the kernel - must be
                // identity-mapped for now
                NebulaMemoryType::Loader => (0, desc.phys_start, PageEntryFlags::default()),
            };

            (0..desc.num_pages).try_for_each(|page| {
                let physical_address = desc.phys_start + page * PAGE_SIZE as u64;
                let virtual_address = virtual_base + physical_base + page * PAGE_SIZE as u64;

                manager.map_memory(virtual_address, physical_address, flags)
            })?;

            Ok(())
        })?;

    // identity map framebuffer (later managed by VMM)
    (0..fb_page_count).try_for_each(|page| {
        let address = fb_base + (page * PAGE_SIZE) as u64;
        manager.map_memory(address, address, PageEntryFlags::default_nx())
    })?;

    // todo: fix bug that causes this to freeze on some machines
    // enable no-execute feature if available
    if let Some(mut efer) = Efer::read() {
        nx = true;
        efer.insert(Efer::NXE);
        efer.write();
    }

    // update bootinfo values
    unsafe {
        graphics::logger::update_font(KERNEL_DATA_VIRTUAL - first_data_addr);
        bootinfo_ref.mmap.descriptors = (memory_map.descriptors as u64 + KERNEL_DATA_VIRTUAL
            - first_data_addr)
            as *mut NebulaMemoryDescriptor;
    }

    // switch to new paging scheme
    unsafe {
        asm!("mov cr3, {0}", in(reg) pml4_addr);
    }

    // update bootinfo pointer (kernel data)
    let bootinfo = (KERNEL_DATA_VIRTUAL + bootinfo as u64 - first_data_addr) as *mut BootInfo;

    // update kernel stack
    let stack = KernelStack {
        bottom: KERNEL_STACK_VIRTUAL,
        top: KERNEL_STACK_VIRTUAL + (KERNEL_STACK_SIZE - PAGE_SIZE) as u64,
        num_pages: old_stack.num_pages,
    };

    // update ptm
    unsafe {
        // offset
        manager.mappings().update_offset(PAS_VIRTUAL);

        // pmm bit map
        manager.pmm().update_bit_map_ptr(PAS_VIRTUAL);

        // pmm memory map
        manager
            .pmm()
            .update_memory_map_ptr(KERNEL_DATA_VIRTUAL - first_data_addr);
    }

    Ok((
        VirtualAddressSpace {
            bootinfo,
            manager,
            stack,
        },
        nx,
    ))
}
