#![no_main]
#![no_std]

use core::panic::PanicInfo;

use log::{error, info};
use qemu_print::qemu_println;
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Hello world!");
    boot::stall(10_000_000);
    Status::SUCCESS
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: \n{:#?}", info);
    qemu_println!("Panic occurred: \n{:#?}", info);
    loop {}
}
