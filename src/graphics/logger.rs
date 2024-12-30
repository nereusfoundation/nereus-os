use core::fmt::Write;
use mem::VirtualAddress;
use framebuffer::{color::Color, raw::write::RawWriter, safe::Writer};
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
    () => {$crate::graphics::logger::_log(format_args!("\n"), FG_COLOR_LOG)};
    ($fg:expr, $($arg:tt)*) => {
         $crate::graphics::logger::_log(format_args!("{}\n", format_args!($($arg)*)), $fg)
    };
}

#[macro_export]
macro_rules! loginfo {
    ($($arg:tt)*) => {
        logln!(FG_COLOR_INFO," [INFO ]: {}", format_args!($($arg)*));
    };
}
#[macro_export]
macro_rules! validate {
    ($result:expr, $msg:expr) => {{
        log!(FG_COLOR_LOG, " [LOG  ]: {}", $msg);
        match $result {
            Ok(value) => {
                logln!(FG_COLOR_OK, " OK");
                value
            }
            Err(err) => {
                logln!();
                log!(FG_COLOR_ERROR, " [ERROR]: ");
                logln!(FG_COLOR_LOG, "{}", err);
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

/// Get Framebuffer Base and page count
///
/// # Safety
/// The caller must guarantee that the logger is avaiable when calling this function.
pub unsafe fn get_fb() -> (u64, usize) {
    let mut locked = LOGGER.locked();
    let writer = locked.get_mut().expect("getting framebuffer requires valid logger");
    writer.fb_meta()
}

/// Take the writer instance out of the logger
pub fn take_writer() -> Option<RawWriter> {
    let mut locked = LOGGER.locked();
    locked.take()
}
