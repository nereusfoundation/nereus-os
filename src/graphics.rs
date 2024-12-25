use framebuffer::{error::FrameBufferError, FrameBuffer};
use uefi::{boot, proto::console::gop::GraphicsOutput};

/// Set up GOP framebuffer
pub(crate) fn initialize_framebuffer() -> Result<FrameBuffer, FrameBufferErrorExt> {
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
        FrameBuffer::new(
            gop.frame_buffer().as_mut_ptr(),
            gop_fb_size,
            gop_mode.resolution().0,
            gop_mode.resolution().1,
            gop_mode.stride(),
            format,
        )
    })
}

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum FrameBufferErrorExt {
    #[error("FrameBuffer error: {0}")]
    FrameBuffer(#[from] FrameBufferError),
    #[error("Uefi error: {0}")]
    Uefi(#[from] uefi::Error),
}
