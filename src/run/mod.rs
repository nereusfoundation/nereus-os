use std::{fs, path::Path, process::Command};

use clap::ValueEnum;

use crate::BootUtilityError;

pub(crate) mod clippy;
pub(crate) mod qemu;
pub(crate) mod usb;

/// Available run options
#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum RunOption {
    Qemu,
    Usb,
    Clippy,
    Clean,
}

/// Removes the os disk-image and the target directory.
pub(crate) fn clean(img_path: &Path) -> Result<(), BootUtilityError> {
    // remove disk image
    fs::remove_file(img_path).map_err(BootUtilityError::from)?;
    // invoke `cargo clean`
    Command::new("cargo")
        .arg("clean")
        .status()
        .map_err(BootUtilityError::from)
        .map(|_| ())
}
