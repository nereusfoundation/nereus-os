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

extern crate alloc;

mod gdt;
mod graphics;
mod idt;
mod io;
mod memory;
mod serial;

#[no_mangle]
pub extern "sysv64" fn _start(bootinfo: &mut BootInfo) -> ! {
    // set up kernel logger
    assert!(
        bootinfo.writer.is_some(),
        "logger must have been initialized in loader"
    );
    LOGGER.initialize(bootinfo.writer.take().unwrap());
    println!(color::CAPTION, " [KERNEL]");
    validate!(
        memory::reclaim_loader_memory(bootinfo).unwrap(),
        "Reclaiming loader memory"
    );

    // todo: tss
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
        memory::kheap::initialize(bootinfo),
        "Initializing kernel heap"
    );

    loginfo!(
        "Heap start address: {:#x}, pages: {:#x}",
        KHEAP_VIRTUAL,
        KHEAP_PAGE_COUNT
    );

    hal::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!(color::ERROR, " [ERROR]: ");
    println!(color::LOG, "Panic occurred: \n{:#?}\n", info);

    serial_println!("Panic ocurred: \n{:#?}\n", info);

    hal::hlt_loop();
}
