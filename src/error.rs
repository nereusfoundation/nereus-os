#[derive(Debug, thiserror::Error)]
pub(crate) enum BootUtilityError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Environment error: {0}")]
    Env(#[from] std::env::VarError),
    #[error("Unabel to find path to executable")]
    CannotFindExec,
    #[error("Invalid image size")]
    InvalidSize,
    #[error("FatFs IO error: {0}")]
    Fatfs(#[from] fatfs::Error<std::io::Error>),
}
