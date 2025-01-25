#![no_std]

pub mod color;
pub mod error;
pub mod raw;

/// Supported Pixel Formats
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PixelFormat {
    Rgb32bit,
    Bgr32bit,
}
