#![no_std]
#![no_main]

use bootinfo::BootInfo;
use core::panic::PanicInfo;
use framebuffer::color::{self};
use graphics::LOGGER;

mod graphics;
mod memory;

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
    hal::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!(color::ERROR, " [ERROR]: ");
    println!(color::LOG, "Panic orccurred: \n{:#?}\n", info);

    hal::hlt_loop();
}
