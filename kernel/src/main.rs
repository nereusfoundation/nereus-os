#![no_std]
#![no_main]

use bootinfo::BootInfo;
use core::{fmt::Write, panic::PanicInfo};

mod memory;

#[no_mangle]
pub extern "sysv64" fn _start(bootinfo: &mut BootInfo) -> ! {
    // todo: set up proper logger
    bootinfo
        .writer
        .write_str(" [INFO ]: Hello nebula kernel!\n")
        .unwrap();

    memory::reclaim_loader_memory(bootinfo).unwrap();

    bootinfo
        .writer
        .write_str("done reclaiming loader mem")
        .unwrap();

    // remap loader memory to avaiable PAS offset mapping
    hlt();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    hlt();
}

fn hlt() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}
