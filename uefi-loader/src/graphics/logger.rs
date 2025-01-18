use core::fmt::Write;
use framebuffer::{color::Color, raw::write::RawWriter};
use hal::interrupts::without_interrupts;
use mem::VirtualAddress;
use sync::locked::Locked;

pub(crate) static LOGGER: Locked<RawWriter> = Locked::new();

#[macro_export]
macro_rules! log {
    ($fg:expr, $($arg:tt)*) => {
         $crate::graphics::logger::_log(format_args!($($arg)*), $fg)
    };
}

#[macro_export]
macro_rules! logln {
    () => {$crate::graphics::logger::_log(format_args!("\n"), framebuffer::color::LOG)};
    ($fg:expr, $($arg:tt)*) => {
         $crate::graphics::logger::_log(format_args!("{}\n", format_args!($($arg)*)), $fg)
    };
}

#[macro_export]
macro_rules! loginfo {
    ($($arg:tt)*) => {
        logln!(framebuffer::color::INFO," [INFO ]: {}", format_args!($($arg)*));
    };
}
#[macro_export]
macro_rules! validate {
    ($result:expr, $msg:expr) => {{
        log!(framebuffer::color::LOG, " [LOG  ]: {}", $msg);
        match $result {
            Ok(value) => {
                logln!(framebuffer::color::OK, " OK");
                value
            }
            Err(err) => {
                logln!();
                log!(framebuffer::color::ERROR, " [ERROR]: ");
                logln!(framebuffer::color::LOG, "{}", err);
                return Status::UNSUPPORTED;
            }
        }
    }};
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

/// Make logger available after switching to new paging scheme
///
/// # Safety
/// Caller must guarantee that the offset is valid. This function must be called with a valid
/// logger available.
pub unsafe fn update_font(offset: VirtualAddress) {
    let mut locked = LOGGER.locked();
    let writer = locked.get_mut().expect("update font requires valid logger");
    let old_ptr = writer.font().glyph_buffer_address();
    let new_ptr = old_ptr as u64 + offset;

    writer.font().update_glyph_buffer_ptr(new_ptr as *const u8);
}

/// Take the writer instance out of the logger
pub fn take_writer() -> Option<RawWriter> {
    let mut locked = LOGGER.locked();
    locked.take()
}
