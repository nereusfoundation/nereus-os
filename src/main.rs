use std::io::ErrorKind;

use clap::Parser;
use cli::Args;
use error::BootUtilityError;
use img::build_img;
use run::{clean, clippy, qemu::QemuConfig, usb, RunOption};

mod cli;
mod error;
mod img;
mod run;

fn main() {
    let args = Args::parse();

    let img = args.img_path;
    let kernel = args.kernel_dir.as_path();
    let loader = args.loader_dir.as_path();

    if matches!(args.run_option, RunOption::Qemu | RunOption::Usb) {
        println!("building boot img - this may take a while...");
        match build_img(kernel, loader, args.font_path.as_path(), img.as_path()) {
            Ok(_) => println!("build complete."),
            Err(err) => {
                eprintln!("build failed - error: {}.", err);
                return;
            }
        }
    }
    match args.run_option {
        RunOption::Usb => match usb::write_img(img, args.usb.unwrap()) {
            Ok(_) => println!("usb ready."),
            Err(err) => {
                eprintln!("usb formatting failed - error: {}.", err);
                if let BootUtilityError::Io(io_err) = err {
                    if io_err.kind() == ErrorKind::PermissionDenied {
                        eprintln!("hint: use `sudo -E <cmd>`")
                    }
                }
            }
        },
        RunOption::Qemu => {
            let qemu = QemuConfig::new(
                args.ovmf_code,
                args.ovmf_vars,
                img,
                args.qemu_log,
                args.qemu_serial,
                args.mem_mb,
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
        RunOption::Clean => match clean(&img) {
            Ok(_) => println!("cleaning complete."),
            Err(err) => eprintln!("cleaning failed - error: {}.", err),
        },
    }
}
