#![no_main]
#![no_std]

use core::panic::PanicInfo;

use graphics::initialize_framebuffer;
use log::{error, info};
use qemu_print::qemu_println;
use uefi::prelude::*;

mod error;
mod graphics;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Bootloader: Hello world!");
    info!("Bootloader: Intializing GOP framebuffer...");
    match initialize_framebuffer() {
        Ok(framebuffer) => {
            for x in 0..framebuffer.width() {
                for y in 0..framebuffer.height() {
                    framebuffer
                        .draw_pixel(
                            x,
                            y,
                            framebuffer::color::Color {
                                red: 0,
                                green: 128,
                                blue: 128,
                            },
                        )
                        .unwrap();
                }
            }
        }
        // this won't always be shown in the console, because stdout may not be avaialable in some cases
        Err(err) => error!("Bootloader: Failed to initialize framebuffer: {}", err),
    }

    boot::stall(10_000_000);
    Status::SUCCESS
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: \n{:#?}", info);
    qemu_println!("Panic occurred: \n{:#?}", info);
    loop {}
}
