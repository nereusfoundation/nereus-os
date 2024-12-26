#![no_main]
#![no_std]

extern crate alloc;

use core::panic::PanicInfo;

use framebuffer::{color::Color, raw::write::RawWriter};
use graphics::{initialize_framebuffer, logger::LOGGER, parse_psf_font};
use log::{error, info};
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

            let writer = RawWriter::new(
                font,
                framebuffer,
                Color::new(255, 255, 255),
                Color::new(0, 0, 0),
            );

            LOGGER.initialize(writer);

            log!("Initialize framebuffer OK")
        }
        // this won't always be shown in the console, because stdout may not be available in some cases
        Err(err) => error!("Bootloader: Failed to initialize framebuffer: {}", err),
    }

    boot::stall(10_000_000);
    Status::SUCCESS
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: \n{:#?}", info);
    log!("Panic orccurred: \n{:#?}", info);
    loop {}
}
