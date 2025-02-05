use core::arch::asm;

pub(crate) mod apic;
pub(crate) mod pic;
pub(crate) mod timer;

/// Write 8 bits to the specified port.
///
/// # Safety
/// Needs IO privileges.
#[inline]
pub(crate) unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value);
    }
}

/// Read 8 bits from the specified port.
///
/// # Safety
/// Needs IO privileges.
#[inline]
pub(crate) unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port);
    value
}

/// Older machines may require to wait a cycle before continuing the io pic communication.
///
/// # Safety
/// Needs IO privileges.
#[inline]
pub(crate) unsafe fn io_wait() {
    asm!("out 0x80, al", in("al") 0u8);
}
