#![no_main]
#![no_std]

extern crate alloc;

use core::{fmt::Write, panic::PanicInfo};

use framebuffer::{color::Color, raw::write::RawWriter};
use graphics::{initialize_framebuffer, parse_psf_font};
use log::{error, info};
use qemu_print::qemu_println;
use uefi::prelude::*;

mod error;
mod file;
mod graphics;

const PSF_FILE_NAME: &str = "font.psf";

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Bootloader: Hello world!");
    info!("Bootloader: Intializing GOP framebuffer...");
    match initialize_framebuffer() {
        Ok(framebuffer) => {
            framebuffer.fill(Color::new(0, 0, 0));
            let font = parse_psf_font(PSF_FILE_NAME).unwrap();

            let mut writer = RawWriter::new(
                font,
                framebuffer,
                Color::new(255, 255, 255),
                Color::new(0, 0, 0),
            );

            writer.write_str("Writer works! :)").unwrap();
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
