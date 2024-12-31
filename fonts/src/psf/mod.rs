use core::slice;

use header::Header;

pub mod header;

pub const PSF1_MAGIC: u16 = 0x0436;
pub const PSF2_MAGIC: u32 = 0x864ab572;

/// Raw PSF Font wrapper that does not provide a safe interface
#[derive(Copy, Clone, Debug)]
pub struct RawFont {
    header: Header,
    glyph_buffer_ptr: *const u8,
    glyph_buffer_size: usize,
}

impl RawFont {
    /// Initialize a new raw PC Screen Font wrapper
    ///
    /// # Safety
    /// Caller must guarantee that the raw font buffer points to valid data.
    pub unsafe fn new(
        header: Header,
        glyph_buffer_ptr: *const u8,
        glyph_buffer_size: usize,
    ) -> RawFont {
        RawFont {
            header,
            glyph_buffer_ptr,
            glyph_buffer_size,
        }
    }
}

impl RawFont {
    /// Update glyph buffer pointer
    ///
    /// # Safety
    /// Caller must guarentee that the new pointer is valid.
    pub unsafe fn update_glyph_buffer_ptr(&mut self, new_ptr: *const u8) {
        self.glyph_buffer_ptr = new_ptr;
    }
}

impl RawFont {
    /// Returns a slice of the font's glyphs
    pub fn glyphs(&self) -> &[u8] {
        // since the buffer address can only be set in the [RawFont::new] function, the font data is guaranteed to be valid
        unsafe { slice::from_raw_parts(self.glyph_buffer_ptr, self.glyph_buffer_size) }
    }

    pub fn glyph_buffer_address(&self) -> *const u8 {
        self.glyph_buffer_ptr
    }

    pub fn glyph_bytes(&self) -> usize {
        match self.header {
            Header::V1(header) => header.character_size as usize,
            Header::V2(header) => header.glyph_size as usize,
        }
    }

    pub fn glyph_height(&self) -> usize {
        match self.header {
            Header::V1(header) => header.character_size as usize,
            Header::V2(header) => header.height as usize,
        }
    }

    pub fn glyph_width(&self) -> usize {
        match self.header {
            Header::V1(_) => 8,
            Header::V2(header) => header.width as usize,
        }
    }
}
