#![no_std]

use core::arch::{global_asm, asm};

global_asm!(include_str!("asm/msr.S"));

pub mod interrupts;
pub mod msr;

#[inline]
pub fn hlt_loop() -> ! {
    loop { unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); } }
}
