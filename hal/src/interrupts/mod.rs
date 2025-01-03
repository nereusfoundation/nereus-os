use core::arch::asm;

use crate::registers::rflags::RFlags;

/// Enable interrupts
#[inline]
pub fn enable() {
    unsafe { asm!("sti", options(preserves_flags, nostack)) }
}

/// Disable interrupts
#[inline]
pub fn disable() {
    unsafe { asm!("cli", options(preserves_flags, nostack)) }
}

/// Whether interrupts are enabled right now
#[inline]
pub fn are_enabled() -> bool {
    RFlags::read().contains(RFlags::INTERRUPTS_ENABLED)
}

/// Execute a block of code without any interruptions
#[inline]
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let were_enabled_flag = are_enabled();

    if were_enabled_flag {
        disable();
    }

    let ret = f();

    if were_enabled_flag {
        enable();
    }

    ret
}
