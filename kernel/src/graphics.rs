use core::fmt::Write;

use framebuffer::{color::Color, raw::write::RawWriter};
use hal::interrupts::without_interrupts;
use sync::locked::Locked;

pub(crate) static LOGGER: Locked<RawWriter> = Locked::new();

#[macro_export]
macro_rules! print {
    ($fg:expr, $($arg:tt)*) => {
         $crate::graphics::_print(format_args!($($arg)*), $fg);
    };
}

#[macro_export]
macro_rules! println {
    () => {$crate::graphics::_print(format_args!("\n"), ::framebuffer::color::LOG)};
    ($fg:expr, $($arg:tt)*) => {
         $crate::graphics::_print(format_args!("{}\n", format_args!($($arg)*)), $fg);
    };
    ($($arg:tt)*) => {
        println!(::framebuffer::color::INFO, $($arg)*);
    }
}

#[macro_export]
macro_rules! loginfo {
    ($($arg:tt)*) => {
        $crate::println!(::framebuffer::color::INFO," [INFO ]: {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::print!(::framebuffer::color::LOG, " [LOG  ]: {}", $($arg)*);
    };
}

#[macro_export]
macro_rules! validate {
    ($fun:stmt, $msg:expr) => {{
        log!($msg);
        $fun();
        println!(::framebuffer::color::OK, " OK");
    }};
    (result $result:expr, $msg:expr) => {{
        log!($msg);
        match $result {
            Ok(value) => {
                println!(::framebuffer::color::OK, " OK");
                value
            }
            Err(err) => {
                println!();
                print!(::framebuffer::color::ERROR, " [ERROR]: ");
                println!(::framebuffer::color::LOG, "{}", err);
                unimplemented!("error recovery");
            }
        }
    }};
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments, fg: Color) {
    without_interrupts(|| {
        if let Some(writer) = LOGGER.locked().get_mut() {
            let (old_fg, old_bg) = writer.colors();

            writer.set_colors(fg, old_bg);

            writer.write_fmt(args).unwrap();

            writer.set_colors(old_fg, old_bg);
        }
    });
}
