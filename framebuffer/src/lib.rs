#![no_std]

pub mod color;
pub mod error;
pub mod raw;

pub const BYTES_PER_PIXEL: usize = 4;

/// Supported Pixel Formats
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PixelFormat {
    Rgb,
    Bgr,
}
