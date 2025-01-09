use bitflags::bitflags;
use core::{cell::LazyCell, fmt};
use sync::spin::SpinLock;
use uart::SerialPort;

static PORT: SpinLock<LazyCell<SerialPort>> = SpinLock::new(LazyCell::new(|| {
    let mut port = unsafe { SerialPort::new(0x3f8) };

    port.init();

    port
}));

// Based on the uart_16550 crate
pub(crate) mod macros;
pub(crate) mod uart;

bitflags! {
    /// Interrupt enable flags
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct IntEnFlags: u8 {
        const RECEIVED = 1;
        const SENT = 1 << 1;
        const ERRORED = 1 << 2;
        const STATUS_CHANGE = 1 << 3;
        // 4 to 7 are unused
    }
}

bitflags! {
    /// Line status flags
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5;
        // 6 and 7 unknown
    }
}

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WouldBlockError;

impl fmt::Display for WouldBlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("serial device not ready")
    }
}
