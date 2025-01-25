use core::ptr::slice_from_raw_parts_mut;

use crate::PixelFormat;

pub mod draw;
pub mod write;

/// FrameBuffer metadata struct
#[derive(Copy, Clone, Debug)]
pub struct RawFrameBuffer {
    /// Pointer to address of framebuffer
    ptr: *mut [u8],
    /// Width of framebuffer in pixels
    width: usize,
    /// Height of framebuffer in pixels
    height: usize,
    /// Pixels per scanline
    stride: usize,
    /// Pixel format of framebuffer
    format: PixelFormat,
    /// Number of bytes per pixel. Commonly 4.
    bpp: usize,
}

impl RawFrameBuffer {
    /// Initiate a new FrameBuffer wrapper.
    ///
    /// # Safety
    /// Caller must guarantee that the attribute are valid.
    pub unsafe fn new(
        ptr: *mut u8,
        size: usize,
        width: usize,
        height: usize,
        stride: usize,
        format: PixelFormat,
        bpp: usize,
    ) -> RawFrameBuffer {
        let ptr = slice_from_raw_parts_mut(ptr, size);

        RawFrameBuffer {
            ptr,
            width,
            height,
            stride,
            format,
            bpp,
        }
    }
}

impl RawFrameBuffer {
    /// Get unsafe mutable ptr to address of framebuffer
    pub fn ptr(&self) -> *mut [u8] {
        self.ptr
    }

    /// Get width of framebuffer in pixels
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get height of framebuffer in pixels
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get pixels per scanline
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Get [PixelFormat] of framebuffer
    pub fn format(&self) -> PixelFormat {
        self.format
    }
}

impl RawFrameBuffer {
    /// Update the address of the framebuffer
    ///
    /// # Safety
    /// The caller must ensure that the address is valid and mapped.
    pub unsafe fn update_ptr(&mut self, address: *mut u8) {
        let len = self.ptr().len();
        self.ptr = slice_from_raw_parts_mut(address, len)
    }
}
