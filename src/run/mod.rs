pub(crate) mod clippy;
pub(crate) mod qemu;
pub(crate) mod usb;

pub(crate) enum RunOption {
    Qemu,
    Usb,
    Clippy,
}
