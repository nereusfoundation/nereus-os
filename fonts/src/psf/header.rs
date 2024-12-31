#[derive(Copy, Clone, Debug)]
pub enum Header {
    V1(PSF1Header),
    V2(PSF2Header),
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PSF1Header {
    /// Magic number: 0x0436
    pub magic: u16,
    /// Font Mode: Whether font is a 256 or 512 glyph set
    pub font_mode: u8,
    /// Character Size: Glyph height
    pub character_size: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PSF2Header {
    /// Magic number: 0x864ab572
    pub magic: u32,
    /// Version: currently always 0
    pub version: u32,
    /// Header Size: Size of header in bytes (usually 32)
    pub header_size: u32,
    // Flags: Indicate unicode table (0 if there isn't one)
    pub flags: u32,
    /// Length: Number of glyphs
    pub length: u32,
    /// Glyph Size: Number of bytes per glyph
    pub glyph_size: u32,
    /// Height: Height of each glyph
    pub height: u32,
    /// Width: Width of each glyph
    pub width: u32,
}
