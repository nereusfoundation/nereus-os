use fonts::psf::{
    header::{Header, PSF1Header, PSF2Header},
    RawFont, PSF1_MAGIC, PSF2_MAGIC,
};
use framebuffer::{color::Color, raw::RawFrameBuffer};
use uefi::{boot, proto::console::gop::GraphicsOutput};

use crate::{
    error::{FrameBufferErrorExt, PsfParseError},
    file,
};

pub(crate) mod logger;

pub(crate) const FG_COLOR_INFO: Color = Color::new(255, 255, 255);
pub(crate) const FG_COLOR_ERROR: Color = Color::new(255, 0, 0);
pub(crate) const FG_COLOR_OK: Color = Color::new(0, 255, 100);
pub(crate) const BG_COLOR: Color = Color::new(0, 0, 0);

/// Set up GOP framebuffer
pub(crate) fn initialize_framebuffer() -> Result<RawFrameBuffer, FrameBufferErrorExt> {
    let handle =
        boot::get_handle_for_protocol::<GraphicsOutput>().map_err(FrameBufferErrorExt::from)?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(handle)
        .map_err(FrameBufferErrorExt::from)?;

    let gop_mode = gop.current_mode_info();
    let gop_fb_size = gop.frame_buffer().size();
    let format = match gop_mode.pixel_format() {
        uefi::proto::console::gop::PixelFormat::Rgb => framebuffer::PixelFormat::Rgb,
        uefi::proto::console::gop::PixelFormat::Bgr => framebuffer::PixelFormat::Bgr,
        uefi::proto::console::gop::PixelFormat::Bitmask => unimplemented!(),
        uefi::proto::console::gop::PixelFormat::BltOnly => unimplemented!(),
    };

    Ok(unsafe {
        RawFrameBuffer::new(
            gop.frame_buffer().as_mut_ptr(),
            gop_fb_size,
            gop_mode.resolution().0,
            gop_mode.resolution().1,
            gop_mode.stride(),
            format,
        )
    })
}

/// Load PSF font into memory
pub(crate) fn parse_psf_font(fontname: &'static str) -> Result<RawFont, PsfParseError> {
    let font_data = file::get_file_data(fontname).map_err(PsfParseError::from)?;
    let font_data_ptr = font_data.as_ptr(); // points to first byte of font data

    // check for sufficient font length for psf header
    if font_data.len() < size_of::<PSF1Header>() {
        return Err(PsfParseError::InsufficientDataForPSFHeader);
    }

    let magic = unsafe { *(font_data_ptr as *const u16) };

    // check for psf1 header magic
    if magic == PSF1_MAGIC {
        let header = unsafe { *(font_data_ptr as *const PSF1Header) };
        let glyph_buffer_length = if header.font_mode == 1 { 512 } else { 256 };
        let glyph_buffer_size = glyph_buffer_length * header.character_size as usize;

        let total_size = size_of::<PSF1Header>() + glyph_buffer_size;

        // check for sufficient font length for psf 1 data
        if font_data.len() < total_size {
            return Err(PsfParseError::InsufficientDataForPSF1);
        }

        // allocate memory for entire font data
        let font_address = boot::allocate_pool(boot::MemoryType::LOADER_DATA, total_size)
            .map_err(PsfParseError::from)?;

        // copy header data to allocated memory
        unsafe {
            core::ptr::copy_nonoverlapping(
                font_data_ptr,
                font_address.as_ptr(),
                size_of::<PSF1Header>(),
            );
        }

        // copy font data to allocated memory
        let glyph_buffer_ptr = unsafe { (font_address.as_ptr()).add(size_of::<PSF1Header>()) };

        unsafe {
            core::ptr::copy_nonoverlapping(
                font_data_ptr.add(size_of::<PSF1Header>()),
                glyph_buffer_ptr,
                glyph_buffer_size,
            );
        }

        Ok(unsafe {
            RawFont::new(
                Header::V1(header),
                glyph_buffer_ptr as *const u8,
                glyph_buffer_length,
            )
        })
    } else {
        // check for psf2 header magic
        let magic = unsafe { *(font_data_ptr as *const u32) };

        if magic != PSF2_MAGIC {
            return Err(PsfParseError::InvalidPSFMagic(magic));
        }

        let header = unsafe { *(font_data_ptr as *const PSF2Header) };

        let glyph_buffer_size = (header.length * header.glyph_size) as usize;
        let total_size = size_of::<PSF2Header>() + glyph_buffer_size;

        // check for sufficient font length for psf 2 data
        if font_data.len() < total_size {
            return Err(PsfParseError::InsufficientDataForPSF2);
        }

        // allocate memory for entire font data
        let font_address = boot::allocate_pool(boot::MemoryType::LOADER_DATA, total_size)
            .map_err(PsfParseError::from)?;

        // copy header data to allocated memory
        unsafe {
            core::ptr::copy_nonoverlapping(
                font_data_ptr,
                font_address.as_ptr(),
                size_of::<PSF2Header>(),
            );
        }

        // copy font data to allocated memory
        let glyph_buffer_ptr = unsafe { (font_address.as_ptr()).add(size_of::<PSF2Header>()) };

        unsafe {
            core::ptr::copy_nonoverlapping(
                font_data_ptr.add(size_of::<PSF2Header>()),
                glyph_buffer_ptr,
                glyph_buffer_size,
            );
        }

        Ok(unsafe {
            RawFont::new(
                Header::V2(header),
                glyph_buffer_ptr as *const u8,
                header.length as usize,
            )
        })
    }
}
