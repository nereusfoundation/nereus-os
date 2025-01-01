#![feature(vec_into_raw_parts)]
#![no_main]
#![no_std]

extern crate alloc;

use core::{arch::asm, panic::PanicInfo};

use alloc::vec::Vec;
use boot::MemoryType;
use framebuffer::{color::Color, raw::write::RawWriter};
use graphics::{
    initialize_framebuffer,
    logger::{self, LOGGER},
    parse_psf_font, BG_COLOR, CAPTION, FG_COLOR_CAPTION, FG_COLOR_ERROR, FG_COLOR_INFO,
    FG_COLOR_LOG, FG_COLOR_OK,
};
use log::{error, info};
use mem::{bitmap_allocator::BitMapAllocator, KERNEL_STACK_SIZE, PAGE_SIZE};
use memory::{
    NebulaMemoryDescriptor, NebulaMemoryMap, NebulaMemoryType, KERNEL_CODE, KERNEL_DATA,
    KERNEL_STACK, MMAP_META_DATA, PSF_DATA,
};
use uefi::{mem::memory_map::MemoryMap, prelude::*};

mod error;
mod file;
mod graphics;
mod memory;

const PSF_FILE_NAME: &str = "font.psf";
const KERNEL_FILE_NAME: &str = "kernel.elf";

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Bootloader: Hello world!");
    info!("Bootloader: Intializing GOP framebuffer...");
    match initialize_framebuffer() {
        Ok(framebuffer) => {
            framebuffer.fill(Color::new(0, 0, 0));
            let font = parse_psf_font(PSF_FILE_NAME).unwrap();

            let fb_addr = framebuffer.ptr() as *mut u8 as u64;
            let fb_page_num = framebuffer.ptr().len().div_ceil(PAGE_SIZE);

            let writer = RawWriter::new(font, framebuffer, FG_COLOR_LOG, BG_COLOR);

            LOGGER.initialize(writer);

            logln!(FG_COLOR_CAPTION, "{}", CAPTION);

            log!(FG_COLOR_LOG, " [LOG  ]: Initializing framebuffer ");
            logln!(FG_COLOR_OK, "OK");

            loginfo!(
                "Framebuffer address: {:#x}, pages: {:#x}",
                fb_addr,
                fb_page_num
            );

            // get kernel file from disk
            let kernel_data = validate!(
                file::get_file_data(KERNEL_FILE_NAME),
                "Retrieving kernel file image"
            );

            loginfo!("Kernel size: {} bytes", kernel_data.len());

            let kernel_elf = validate!(
                file::elf::Elf::try_new(kernel_data),
                "Loading kernel image into memory"
            );

            loginfo!(
                "Kernel entry: {:#x}, file base: {:#x}, pages: {:#x}",
                kernel_elf.entry(),
                kernel_elf.base(),
                kernel_elf.num_pages()
            );

            let kernel_stack = validate!(
                memory::stack::allocate_kernel_stack(KERNEL_STACK_SIZE),
                "Allocating memory for kernel stack"
            );
            loginfo!(
                "Kernel stack top: {:#x}, bottom: {:#x}, pages: {:#x}",
                kernel_stack.top(),
                kernel_stack.bottom(),
                kernel_stack.num_pages()
            );

            let (bootinfo_ptr, mmap_descriptors) = validate!(
                memory::bootinfo::allocate_bootinfo(),
                "Allocating memory for kernel bootinfo"
            );

            loginfo!(
                "Kernel boot info address: {:#x}",
                bootinfo_ptr.as_ptr() as u64
            );

            log!(FG_COLOR_LOG, " [LOG  ]: Exiting boot services ");
            let memory_map = drop_boot_services(mmap_descriptors);
            logln!(FG_COLOR_OK, "OK");

            // set memory map of boot info to the correct one
            unsafe {
                bootinfo_ptr
                    .as_ptr()
                    .as_mut()
                    .expect("bootinfo ptr must be valid")
                    .mmap = memory_map;
            }

            let mut pmm = validate!(
                BitMapAllocator::try_new(memory_map),
                "Initializing physical memory manager"
            );
            loginfo!(
                "BitMapAllocator address: {:#x}, page count: {:#x}",
                pmm.address(),
                pmm.pages()
            );
            loginfo!("Free memory: {} bytes", pmm.free_memory());
            loginfo!("Used memory: {} bytes", pmm.used_memory());
            loginfo!("Reserved memory: {} bytes", pmm.reserved_memory());

            log!(
                FG_COLOR_LOG,
                " [LOG  ]: Initializing higher-half kernel address space "
            );

            let (vas, nx) = memory::initialize_address_space(
                bootinfo_ptr.as_ptr(),
                pmm,
                kernel_stack,
                fb_addr,
                fb_page_num,
            )
            .expect("Error during `initialize_address_space`");

            logln!(FG_COLOR_OK, "OK");
            loginfo!("Switchted to kernel page mappings");
            if nx {
                loginfo!("Enabled NO-EXECUTE CPU feature");
            }
            loginfo!("Handing control to kernel...");

            let bootinfo_ref = unsafe { vas.bootinfo.as_mut().unwrap() };

            // assign ptm to bootinfo
            bootinfo_ref.ptm = vas.manager;

            // assign writer to bootinfo
            bootinfo_ref.writer = logger::take_writer().unwrap();

            unsafe {
                asm!( "mov rdi, {1}", "mov rsp, {2}", "jmp {0}", in(reg) kernel_elf.entry(), in(reg) vas.bootinfo,  in(reg) vas.stack.top(), options(noreturn));
            }
        }
        // this won't always be shown in the console, because stdout may not be available in some cases
        Err(err) => error!("Bootloader: Failed to initialize framebuffer: {}", err),
    }

    hal::hlt_loop();
}

fn drop_boot_services(mut mmap_descriptors: Vec<NebulaMemoryDescriptor>) -> NebulaMemoryMap {
    let mmap = unsafe { boot::exit_boot_services(KERNEL_DATA) };

    let mut first_addr = u64::MAX;
    let mut first_available_addr = first_addr;
    let mut last_addr = u64::MIN;
    let mut last_available_addr = last_addr;

    // convert memory map
    mmap.entries().for_each(|desc| {
        let phys_end = desc.phys_start + desc.page_count * PAGE_SIZE as u64;

        if desc.phys_start < first_addr {
            first_addr = desc.phys_start;
        }

        if phys_end > last_addr {
            last_addr = phys_end;
        }

        let r#type = if desc.phys_start < 0x1000 {
            NebulaMemoryType::Reserved
        } else {
            match desc.ty {
                MemoryType::CONVENTIONAL | MMAP_META_DATA => NebulaMemoryType::Available,
                KERNEL_CODE => NebulaMemoryType::KernelCode,
                KERNEL_DATA | PSF_DATA => NebulaMemoryType::KernelData,
                KERNEL_STACK => NebulaMemoryType::KernelStack,
                MemoryType::ACPI_RECLAIM | MemoryType::ACPI_NON_VOLATILE => {
                    NebulaMemoryType::AcpiData
                }
                MemoryType::LOADER_CODE
                | MemoryType::LOADER_DATA
                | MemoryType::BOOT_SERVICES_CODE
                | MemoryType::BOOT_SERVICES_DATA => NebulaMemoryType::Loader,
                _ => NebulaMemoryType::Reserved,
            }
        };

        if desc.phys_start < first_available_addr && r#type == NebulaMemoryType::Available {
            first_available_addr = desc.phys_start;
        }

        if phys_end > last_available_addr && r#type == NebulaMemoryType::Available {
            last_available_addr = phys_end;
        }

        mmap_descriptors.push(NebulaMemoryDescriptor {
            phys_start: desc.phys_start,
            phys_end,
            num_pages: desc.page_count,
            r#type,
        });
    });

    let (ptr, len, _cap) = mmap_descriptors.into_raw_parts();
    NebulaMemoryMap {
        descriptors: ptr,
        descriptors_len: len as u64,
        first_addr,
        first_available_addr,
        last_addr,
        last_available_addr,
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log!(FG_COLOR_ERROR, " [ERROR]: ");
    logln!(FG_COLOR_LOG, "Panic orccurred: \n{:#?}", info);
    hal::hlt_loop();
}
