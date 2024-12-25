use framebuffer::error::FrameBufferError;

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum FrameBufferErrorExt {
    #[error("FrameBuffer error: {0}")]
    FrameBuffer(#[from] FrameBufferError),
    #[error("Uefi error: {0}")]
    Uefi(#[from] uefi::Error),
}
