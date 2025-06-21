use framebuffer::error::FrameBufferError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum FrameBufferErrorExt {
    #[error("FrameBuffer error: {0}")]
    FrameBuffer(#[from] FrameBufferError),
    #[error("Uefi error: {0}")]
    Uefi(#[from] uefi::Error),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PsfParseError {
    #[error("Insufficient font data for PSF header")]
    InsufficientDataForPSFHeader,
    #[error("Insufficient font data for PSF1 data")]
    InsufficientDataForPSF1,
    #[error("Insufficient font data for PSF2 data")]
    InsufficientDataForPSF2,
    #[error("Unrecognized PSF header magic: {0}")]
    InvalidPSFMagic(u32),
    #[error("Uefi error: {0}")]
    Uefi(#[from] uefi::Error),
    #[error("File parsing failed: {0}")]
    File(#[from] FileParseError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum FileParseError {
    #[error("Uefi error: {0}")]
    Uefi(#[from] uefi::Error),
    #[error("Uefi Filesystem error: {0}")]
    UefiFs(#[from] uefi::fs::Error),
    #[error("Invalid filename: {0}")]
    InvalidFile(&'static str),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ElfParseError {
    #[error("Uefi error: {0}")]
    Uefi(#[from] uefi::Error),
    #[error("Elf parsing error: {0}")]
    Goblin(#[from] goblin::error::Error),
    #[error("Invalid ELF-format, 64-bit is required")]
    InvalidFormat,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RsdpError {
    #[error("RSDP table address cannot be found")]
    NotFound,
}
