#![no_std]
#![no_main]
#![feature(lazy_get)]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(once_cell_get_mut)]

use bootinfo::BootInfo;
use core::panic::PanicInfo;
use framebuffer::color::{self};
use graphics::LOGGER;
use mem::{KHEAP_PAGE_COUNT, KHEAP_VIRTUAL};
use memory::vmm::{self, paging::PTM};

extern crate alloc;

mod acpi;
mod gdt;
mod graphics;
mod idt;
mod io;
mod memory;
mod serial;

#[no_mangle]
pub extern "sysv64" fn _start(bootinfo: &mut BootInfo) -> ! {
    // set up global control structures
    LOGGER.initialize(
        bootinfo
            .writer
            .take()
            .expect("logger must have been initialized in loader"),
    );
    println!(color::CAPTION, "\n [KERNEL]");

    validate!(
        PTM.initialize(
            bootinfo
                .ptm
                .take()
                .expect("ptm must have been initialized in loader"),
        ),
        "Reinitializing page table manager"
    );

    validate!(result
        memory::vmm::paging::reclaim_loader_memory(bootinfo),
        "Reclaiming loader memory"
    );

    validate!(
        unsafe {
            gdt::load();
        },
        "Loading global descriptor table"
    );
    validate!(
        unsafe {
            idt::load();
        },
        "Loading interrupt descriptor table"
    );

    validate!(result
        memory::kheap::initialize(),
        "Initializing kernel heap"
    );

    loginfo!(
        "Heap start address: {:#x}, pages: {:#x}",
        KHEAP_VIRTUAL,
        KHEAP_PAGE_COUNT
    );

    validate!(result
        unsafe { vmm::initialize() },
        "Initializing virtual memory manager"
    );

    validate!(result
         memory::vmm::paging::remap_framebuffer(), 
         "Remapping framebuffer as MMIO");

    validate!(result
        acpi::parse(bootinfo.rsdp), "Parsing ACPI tables");

    validate!(result memory::vmm::paging::reclaim_acpi_memory(bootinfo.mmap), "Reclaiming ACPI memory");

    validate!(result io::apic::initialize(), "Initializing advanced programmable interrupt controller");

    hal::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!(color::ERROR, " [ERROR]: ");
    println!(color::LOG, "Panic occurred: \n{:#?}\n", info);

    serial_println!("Panic ocurred: \n{:#?}\n", info);

    hal::hlt_loop();
}
