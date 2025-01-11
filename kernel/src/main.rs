#![no_std]
#![no_main]
#![feature(lazy_get)]
#![feature(fn_align)]
#![feature(naked_functions)]

use bootinfo::BootInfo;
use core::{arch::asm, panic::PanicInfo};
use framebuffer::color::{self};
use graphics::LOGGER;

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
    log!("Reclaiming loader memory ");
    memory::reclaim_loader_memory(bootinfo).unwrap();
    println!(color::OK, "OK");
    log!("Loading global descriptor table ");
    unsafe {
        gdt::load();
    }
    println!(color::OK, "OK");

    log!("Loading interrupt descriptor table ");
    unsafe {
        idt::load();
    }

    println!(color::OK, "OK");
    unsafe {
        let ptr: *const u8 = 0xdeadbeef as *const u8;

        serial_println!("Value at invalid address: {}", *ptr);
    }

    loginfo!("Returned from IDT!");
    hal::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!(color::ERROR, " [ERROR]: ");
    println!(color::LOG, "Panic occurred: \n{:#?}\n", info);

    serial_println!("Panic ocurred: \n{:#?}\n", info);

    hal::hlt_loop();
}
