use std::path::PathBuf;

use clap::Parser;

use crate::RunOption;

/// NereusOS Boot Utility. Used for testing and disk creation.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, author)]
pub(super) struct Args {
    /// Kernel cargo project directory path
    #[arg(long, default_value = "kernel")]
    pub kernel_dir: PathBuf,

    /// UEFI loader cargo project directory path
    #[arg(long, default_value = "uefi-loader")]
    pub loader_dir: PathBuf,

    /// Path to the font file
    #[arg(long, default_value = "psf/light16.psf")]
    pub font_path: PathBuf,

    /// Path to the OS image file
    #[arg(long, default_value = "nereus-os.img")]
    pub img_path: PathBuf,

    /// Path to the OVMF_CODE firmware file
    #[arg(long, default_value = "/usr/share/OVMF/x64/OVMF_CODE.4m.fd")]
    pub ovmf_code: PathBuf,

    /// Path to the OVMF_VARS firmware file
    #[arg(long, default_value = "/usr/share/OVMF/x64/OVMF_VARS.4m.fd")]
    pub ovmf_vars: PathBuf,

    /// Path to the QEMU log file
    #[arg(long, default_value = "qemu.log")]
    pub qemu_log: PathBuf,

    /// QEMU serial output configuration
    #[arg(long, default_value = "file:stdio.log")]
    pub qemu_serial: PathBuf,

    /// Amount of memory (MB) to allocate
    #[arg(long, default_value_t = 512)]
    pub mem_mb: u64,
    /// USB device path (required only if `run_option` is `usb`)
    #[arg(long, required_if_eq("run_option", "usb"))]
    pub usb: Option<PathBuf>,
    /// Run option (qemu, usb, or clippy, clean)
    #[arg(long, value_enum, default_value_t = RunOption::Qemu)]
    pub run_option: RunOption,
    /// Build in release mode (`cargo build --release`)
    #[arg(long, short, action)]
    pub release: bool,
}
