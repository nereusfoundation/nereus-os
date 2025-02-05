use core::cell::LazyCell;

use hal::interrupts::without_interrupts;

use crate::serial::{uart::SerialPort, PORT};

#[macro_export]
macro_rules! retry_until_ok {
    ($cond:expr) => {
        loop {
            if let Ok(ok) = $cond {
                break ok;
            }
            core::hint::spin_loop();
        }
    };
}
#[doc(hidden)]
pub(crate) fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    without_interrupts(|| {
        let mut locked = PORT.lock();
        let port = LazyCell::<SerialPort>::force_mut(&mut locked);
        port.write_fmt(args).expect("Printing to serial failed");
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::macros::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
