#![no_std]

use core::arch::asm;

pub mod interrupts;
pub mod registers;

#[inline]
pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
