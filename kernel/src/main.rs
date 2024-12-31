#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};
use bootinfo::BootInfo;

#[no_mangle]
pub extern "sysv64" fn _start(bootinfo: &mut BootInfo) -> ! {
    bootinfo.writer.write_str(" [INFO ]: Hello nebula kernel!").unwrap();
    // todo: unmap & unreserve loader memory
    hlt();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    hlt();
}

fn hlt() -> !{
    loop { unsafe { core::arch::asm!("hlt") } }
}
