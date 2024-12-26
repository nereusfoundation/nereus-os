use core::fmt::Write;
use framebuffer::{color::Color, safe::Writer};
use hal::interrupts::without_interrupts;

pub(crate) static LOGGER: Writer = Writer::new();

#[macro_export]
macro_rules! log {
    ($fg:expr, $($arg:tt)*) => {
         $crate::graphics::logger::_log(format_args!($($arg)*), $fg)
    };
}

#[macro_export]
macro_rules! logln {
    () => {$crate::graphics::logger::_log(format_args!("\n"), FG_COLOR_INFO)};
    ($fg:expr, $($arg:tt)*) => {
         $crate::graphics::logger::_log(format_args!("{}\n", format_args!($($arg)*)), $fg)
    };
}

#[macro_export]
macro_rules! validate {
    ($result:expr, $msg:expr) => {
        log!(FG_COLOR_INFO, " [LOG  ]: {}", $msg);
        if let Err(err) = $result {
            logln!();
            log!(FG_COLOR_ERROR, " [ERROR]: ");
            logln!(FG_COLOR_INFO, "{}", err);
            return Status::UNSUPPORTED;
        }

        logln!(FG_COLOR_OK, " OK");
    };
}

#[doc(hidden)]
pub fn _log(args: core::fmt::Arguments, fg: Color) {
    without_interrupts(|| {
        if let Some(writer) = LOGGER.locked().get_mut() {
            let (old_fg, old_bg) = writer.colors();

            writer.set_colors(fg, old_bg);

            writer.write_fmt(args).unwrap();

            writer.set_colors(old_fg, old_bg);
        }
    });
}
