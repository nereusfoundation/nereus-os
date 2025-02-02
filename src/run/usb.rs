use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use crate::error::BootUtilityError;

/// Write the given disk file to the specified USB device.
pub(crate) fn write_img(img: PathBuf, usb: PathBuf) -> Result<(), BootUtilityError> {
    let mut img = File::open(img).map_err(BootUtilityError::from)?;

    let mut usb = OpenOptions::new()
        .write(true)
        .open(usb)
        .map_err(BootUtilityError::from)?;

    // buffer for data tranferal (4KB)
    let mut buffer = [0; 4096];

    // copy image to usb
    let mut bytes_read = img.read(&mut buffer).map_err(BootUtilityError::from)?;
    while bytes_read > 0 {
        usb.write(&buffer).map_err(BootUtilityError::from)?;
        bytes_read = img.read(&mut buffer).map_err(BootUtilityError::from)?;
    }
    Ok(())
}
