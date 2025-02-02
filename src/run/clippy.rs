use std::{path::Path, process::Command};

use crate::error::BootUtilityError;

/// Executes the clippy command for the cargo project specified by it's path
fn run(project: &Path) -> Result<(), BootUtilityError> {
    Command::new("cargo")
        .current_dir(project)
        .arg("clippy")
        .status()
        .map_err(BootUtilityError::from)
        .map(|_| ())
}

/// Executes the clippy command for the kernel and loader projects
pub(crate) fn all(kernel: &Path, loader: &Path) -> Result<(), BootUtilityError> {
    run(kernel)?;
    run(loader)
}
