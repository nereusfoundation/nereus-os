use std::path::{Path, PathBuf};

use img::build_img;
use run::{clippy, qemu::QemuConfig, usb, RunOption};

// todo: proper clap interface
const KERNEL_DIR: &str = "kernel";
const LOADER_DIR: &str = "uefi-loader";
const FONT_PATH: &str = "psf/light16.psf";
const IMG_PATH: &str = "nereus-os.img";

const OVMF_CODE: &str = "/usr/share/OVMF/x64/OVMF_CODE.4m.fd";
const OVMF_VARS: &str = "/usr/share/OVMF/x64/OVMF_VARS.4m.fd";
const QEMU_LOG: &str = "qemu.log";
const QEMU_SERIAL: &str = "file:stdio.log";
const MEM_MB: u64 = 512;

const USB: &str = "/dev/sda";

mod error;
mod img;
mod run;

fn main() {
    let opt = RunOption::Qemu;

    let img = PathBuf::from(IMG_PATH);
    let kernel = Path::new(KERNEL_DIR);
    let loader = Path::new(LOADER_DIR);

    if matches!(opt, RunOption::Qemu | RunOption::Usb) {
        match build_img(kernel, loader, Path::new(FONT_PATH), img.as_path()) {
            Ok(_) => println!("build complete."),
            Err(err) => {
                eprintln!("build failed - error: {}.", err);
                return;
            }
        }
    }
    match opt {
        RunOption::Usb => match usb::write_img(img, PathBuf::from(USB)) {
            Ok(_) => println!("usb ready."),
            Err(err) => eprintln!(
                "usb formatting failed - error: {}. Hint: use `sudo -E cargo run`",
                err
            ),
        },
        RunOption::Qemu => {
            let qemu = QemuConfig::new(
                PathBuf::from(OVMF_CODE),
                PathBuf::from(OVMF_VARS),
                img,
                PathBuf::from(QEMU_LOG),
                PathBuf::from(QEMU_SERIAL),
                MEM_MB,
            );

            match qemu.run() {
                Ok(_) => println!("emulation complete."),
                Err(err) => eprintln!("emulation failed - error: {}.", err),
            }
        }
        RunOption::Clippy => match clippy::all(kernel, loader) {
            Ok(_) => println!("clippy invocation complete."),
            Err(err) => eprintln!("clippy failed - error: {}.", err),
        },
    }
}
