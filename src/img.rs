use std::{
    env,
    fs::{File, OpenOptions},
    io,
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::BootUtilityError;
use fatfs::{format_volume, FormatVolumeOptions, ReadWriteSeek};
use serde_json::Value;
const FAT32_OVERHEAD: u64 = 1024 * 1024; // 1 MB
const DIR_OVERHEAD: u64 = 4 * 1024; // 4KB

/// Builds the cargo project at the given path and returns the executable file as well as
/// it's size.
fn build(path: PathBuf) -> Result<(std::fs::File, u64), BootUtilityError> {
    // move to project dir
    env::set_current_dir(&path).map_err(BootUtilityError::from)?;

    let out = Command::new("cargo")
        .arg("build")
        .arg("--message-format=json")
        .output()
        .map_err(BootUtilityError::from)?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let path = stdout
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|val| val["executable"].as_str().map(String::from))
        .next()
        .ok_or(BootUtilityError::CannotFindExec)
        .map(PathBuf::from)?;
    let f = File::open(path).map_err(BootUtilityError::from)?;
    let size = f.metadata().map_err(BootUtilityError::from)?.size();
    Ok((f, size))
}

/// Calculates the total size of the image in bytes. Aligned to the next full MB.
fn calc_img_size(kernel: u64, loader: u64, font: u64) -> Result<u64, BootUtilityError> {
    // "/", "/efi", "/efi/boot"
    let num_dir = 3;
    let dirs = num_dir * DIR_OVERHEAD;

    let total_size = kernel + loader + font + dirs + FAT32_OVERHEAD;

    // pad to the next MB
    let mb = 1024 * 1024;
    total_size
        .checked_next_multiple_of(mb)
        .ok_or(BootUtilityError::InvalidSize)
}

/// Copies the data from the `std::fs::File` `src` to the specified destination device.
fn copy_data<T>(mut src: File, mut dst: T) -> io::Result<()>
where
    T: ReadWriteSeek,
{
    let mut buffer = [0u8; 4096]; // 4 KB buffer
    let mut bytes_read = src.read(&mut buffer)?;

    while bytes_read > 0 {
        dst.write_all(&buffer[..bytes_read])?;
        bytes_read = src.read(&mut buffer)?;
    }

    Ok(())
}

/// Builds the boot image.
pub(super) fn build_img(
    kernel: &Path,
    loader: &Path,
    font: &Path,
    img: &Path,
) -> Result<(), BootUtilityError> {
    let base: PathBuf = env::var("CARGO_MANIFEST_DIR")
        .map_err(BootUtilityError::from)?
        .into();
    let initial_dir = env::current_dir().map_err(BootUtilityError::from)?;
    let (kernel_original, kernel_size) = build(base.join(kernel))?;
    let (loader_original, loader_size) = build(base.join(loader))?;
    env::set_current_dir(initial_dir).map_err(BootUtilityError::from)?;

    let font_original = File::open(font).map_err(BootUtilityError::from)?;
    let font_size = font_original
        .metadata()
        .map_err(BootUtilityError::from)?
        .size();

    // size in bytes
    let size = calc_img_size(kernel_size, loader_size, font_size)?;

    // create raw image
    let img = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(img)
        .map_err(BootUtilityError::from)?;

    img.set_len(size).map_err(BootUtilityError::from)?;

    // format image
    let mut buf_stream = fscommon::BufStream::new(img);
    format_volume(&mut buf_stream, FormatVolumeOptions::new()).map_err(BootUtilityError::from)?;
    let fs = fatfs::FileSystem::new(buf_stream, fatfs::FsOptions::new())?;

    // create directories and files
    let root = fs.root_dir();
    let efi = root.create_dir("efi").map_err(BootUtilityError::from)?;
    let boot = efi.create_dir("boot").map_err(BootUtilityError::from)?;

    let loader = boot
        .create_file("bootx64.efi")
        .map_err(BootUtilityError::from)?;
    let kernel = root
        .create_file("kernel.elf")
        .map_err(BootUtilityError::from)?;
    let font = root
        .create_file("font.psf")
        .map_err(BootUtilityError::from)?;

    // copy data from original files to image
    copy_data(kernel_original, kernel).map_err(BootUtilityError::from)?;
    copy_data(loader_original, loader).map_err(BootUtilityError::from)?;
    copy_data(font_original, font).map_err(BootUtilityError::from)?;

    Ok(())
}
