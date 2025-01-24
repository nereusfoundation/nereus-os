use core::ptr::write_volatile;

use fonts::psf::RawFont;

use crate::{
    color::Color, error::FrameBufferError, raw::RawFrameBuffer, PixelFormat, BYTES_PER_PIXEL,
};

impl RawFrameBuffer {
    pub fn draw_pixel(&self, x: usize, y: usize, color: Color) -> Result<(), FrameBufferError> {
        if !self.in_bounds(x, y) {
            return Err(FrameBufferError::CoordinatesOutOfBounds(x, y));
        }

        let pitch = self.stride * BYTES_PER_PIXEL;

        unsafe {
            let pixel = (self.ptr as *mut u8).add(pitch * y + BYTES_PER_PIXEL * x);

            match self.format {
                PixelFormat::Rgb => {
                    write_volatile(pixel, color.red()); // Red
                    write_volatile(pixel.add(1), color.green()); // Green
                    write_volatile(pixel.add(2), color.blue()); // Blue
                }
                PixelFormat::Bgr => {
                    write_volatile(pixel, color.blue()); // Blue
                    write_volatile(pixel.add(1), color.green()); // Green
                    write_volatile(pixel.add(2), color.red()); // Red
                }
            }
        }

        Ok(())
    }

    /// Fill the entire display with a certain color
    pub fn fill(&self, color: Color) {
        let pitch = self.stride * BYTES_PER_PIXEL;

        let pixel = match self.format {
            PixelFormat::Rgb => u32::from_ne_bytes([color.red(), color.green(), color.blue(), 255]),
            PixelFormat::Bgr => u32::from_ne_bytes([color.blue(), color.green(), color.red(), 255]),
        };

        for y in 0..self.height {
            for x in 0..self.width {
                unsafe {
                    let ptr: *mut u32 = (self.ptr.cast::<u8>())
                        .add(pitch * y + BYTES_PER_PIXEL * x)
                        .cast();
                    ptr.write_volatile(pixel);
                }
            }
        }
    }
}

impl RawFrameBuffer {
    pub fn draw_char(
        &self,
        character: char,
        x_offset: usize,
        y_offset: usize,
        fg_color: Color,
        bg_color: Color,
        font: RawFont,
    ) -> Result<(), FrameBufferError> {
        if character as usize >= font.glyphs().len() {
            return Err(FrameBufferError::InvalidCharacter);
        }

        let character_offset = character as usize * font.glyph_bytes();
        let character_ptr = unsafe { font.glyph_buffer_address().add(character_offset) };

        let glyph_height = font.glyph_height();
        let glyph_width = font.glyph_width();

        for y in 0..glyph_height {
            for x in 0..glyph_width {
                let byte_index = (y * glyph_width + x) / 8;
                let bit_index = 7 - ((y * glyph_width + x) % 8);

                let byte = unsafe { *character_ptr.add(byte_index) };
                let color = if (byte & (1 << bit_index)) != 0 {
                    fg_color
                } else {
                    bg_color
                };

                self.draw_pixel(x + x_offset, y + y_offset, color)?;
            }
        }

        Ok(())
    }
}

impl RawFrameBuffer {
    fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }
}
