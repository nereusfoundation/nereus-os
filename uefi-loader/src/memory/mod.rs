use core::{
    arch::asm,
    ptr::{self, NonNull},
};

use ::bootinfo::BootInfo;
use hal::registers::msr::{efer::Efer, msr_guard::Msr, ModelSpecificRegister};
use mem::{
    bitmap_allocator::BitMapAllocator,
    error::FrameAllocatorError,
    map,
    paging::{ptm::PageTableManager, PageEntryFlags, PageTable},
    KERNEL_CODE_VIRTUAL, KERNEL_STACK_SIZE, KERNEL_STACK_VIRTUAL, PAGE_SIZE, PAS_VIRTUAL,
    PAS_VIRTUAL_MAX,
};
use stack::KernelStack;
use uefi::boot::MemoryType;

use crate::graphics;

pub(crate) mod bootinfo;
pub(crate) mod stack;

// custom memory types of the NereusLoader
pub(crate) const PSF_DATA: MemoryType = MemoryType::custom(0x8000_0000);
pub(crate) const KERNEL_CODE: MemoryType = MemoryType::custom(0x8000_0001);
pub(crate) const KERNEL_STACK: MemoryType = MemoryType::custom(0x8000_0002);
pub(crate) const KERNEL_DATA: MemoryType = MemoryType::custom(0x8000_0003);
pub(crate) const MMAP_META_DATA: MemoryType = MemoryType::custom(0x8000_0004);

pub(crate) type NereusMemoryMap = map::MemoryMap;
pub(crate) type NereusMemoryDescriptor = map::MemoryDescriptor;
pub(crate) type NereusMemoryType = map::MemoryType;

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
    msr: Option<Msr>,
) -> Result<VirtualAddressSpace, FrameAllocatorError> {
    assert_ne!(bootinfo, ptr::null_mut());
    let bootinfo_ref = unsafe { bootinfo.as_mut().expect("bootinfo ptr must be valid") };
    let mut nx = false;

    if let Some(msr) = msr {
        if Efer::nx_available(msr.get_cpuid()) {
            // Safety: We are in privilege level 0.
            if let Ok(mut efer) = unsafe { Efer::read(msr) } {
                efer.insert(Efer::NXE);
                nx = true;

                // Safety: We are in privilege level 0 and checked Efer::nx_available above.
                unsafe { efer.write_unchecked(msr) };
            }
        }
    }

    let memory_map = bootinfo_ref.mmap;

    let pml4_addr = pmm.request_page()?;
    assert_eq!(
        (pml4_addr as usize) % align_of::<PageTable>(),
        0,
        "pml4 pointer is not aligned"
    );

    let mut pml4 = NonNull::new(pml4_addr as *mut PageTable).unwrap();

    // zero out new table
    unsafe { ptr::write_bytes(pml4.as_mut(), 0, 1) };

    let mut manager = PageTableManager::new(pml4, pmm, nx);

    let nx_flags = manager.nx_flags();

    let first_stack_addr = memory_map
        .descriptors()
        .iter()
        .filter(|desc| desc.r#type == NereusMemoryType::KernelStack)
        .map(|desc| desc.phys_start)
        .min()
        .ok_or(FrameAllocatorError::InvalidMemoryMap)?;

    // map kernel physical address space to canonical higher half (canonical lower half is reserved
    // for userspace)
    memory_map
        .descriptors()
        .iter()
        .try_for_each(|desc| -> Result<(), FrameAllocatorError> {
            let (virtual_base, physical_base, flags) = match desc.r#type {
                // map part of physical address space to higher half
                NereusMemoryType::Available => {
                    if desc.phys_end < PAS_VIRTUAL_MAX {
                        (PAS_VIRTUAL, desc.phys_start, nx_flags)
                    } else {
                        return Ok(());
                    }
                }
                // do not map reserved memory
                NereusMemoryType::Reserved => return Ok(()),
                NereusMemoryType::KernelStack => (
                    KERNEL_STACK_VIRTUAL,
                    desc.phys_start - first_stack_addr,
                    nx_flags,
                ),
                // map kernel data same as available PAS
                NereusMemoryType::KernelData => (PAS_VIRTUAL, desc.phys_start, nx_flags),
                NereusMemoryType::KernelCode => (
                    KERNEL_CODE_VIRTUAL,
                    desc.phys_start,
                    PageEntryFlags::default(),
                ),
                // loader data, code pages will later be reclaimed by the kernel - must be
                // identity-mapped for now
                NereusMemoryType::Loader => (0, desc.phys_start, PageEntryFlags::default()),
                // acpi table will later be reclaimed by the kernel - must be identity-mapped for
                // now
                NereusMemoryType::AcpiData => (0, desc.phys_start, PageEntryFlags::default()),
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
        manager.map_memory(address, address, nx_flags)
    })?;

    // update bootinfo values
    unsafe {
        graphics::logger::update_font(PAS_VIRTUAL);
        bootinfo_ref.mmap.descriptors =
            (memory_map.descriptors as u64 + PAS_VIRTUAL) as *mut NereusMemoryDescriptor;
    }

    // switch to new paging scheme
    unsafe {
        asm!("mov cr3, {0}", in(reg) pml4_addr);
    }

    // update bootinfo pointer (kernel data)
    let bootinfo = (PAS_VIRTUAL + bootinfo as u64) as *mut BootInfo;

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
        manager.pmm().update_memory_map_ptr(PAS_VIRTUAL);
    }

    Ok(VirtualAddressSpace {
        bootinfo,
        manager,
        stack,
    })
}
