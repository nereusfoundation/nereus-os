use clap::ValueEnum;

pub(crate) mod clippy;
pub(crate) mod qemu;
pub(crate) mod usb;

/// Available run options
#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum RunOption {
    Qemu,
    Usb,
    Clippy,
}
