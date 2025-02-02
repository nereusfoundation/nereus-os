use std::{path::PathBuf, process::Command};

use crate::error::BootUtilityError;

#[derive(Debug)]
pub(crate) struct QemuConfig {
    ovmf_code: PathBuf,
    ovmf_vars: PathBuf,
    img: PathBuf,
    log: PathBuf,
    serial: PathBuf,
    /// Qemu Memory (MB)
    memory: u64,
}

impl QemuConfig {
    pub(crate) fn new(
        ovmf_code: PathBuf,
        ovmf_vars: PathBuf,
        img: PathBuf,
        log: PathBuf,
        serial: PathBuf,
        memory: u64,
    ) -> Self {
        QemuConfig {
            ovmf_vars,
            ovmf_code,
            img,
            log,
            serial,
            memory,
        }
    }
}

impl QemuConfig {
    /// Runs the qemu command with the specified config.
    pub(crate) fn run(&self) -> Result<(), BootUtilityError> {
        let mut cmd = Command::new("qemu-system-x86_64");

        cmd.arg("-drive")
            .arg(format!(
                "if=pflash,format=raw,readonly=on,file={}",
                self.ovmf_code.as_os_str().to_string_lossy()
            ))
            .arg("-drive")
            .arg(format!(
                "if=pflash,format=raw,readonly=on,file={}",
                self.ovmf_vars.as_os_str().to_string_lossy()
            ))
            .arg("-drive")
            .arg(format!(
                "format=raw,file={}",
                self.img.as_os_str().to_string_lossy()
            ))
            .arg("-d")
            .arg("int")
            .arg("-D")
            .arg(format!("{}", self.log.as_os_str().to_string_lossy()))
            .arg("-no-reboot")
            .arg("-serial")
            .arg(format!("{}", self.serial.as_os_str().to_string_lossy()))
            .arg("-m")
            .arg(format!("{}M", self.memory));

        cmd.spawn().map_err(BootUtilityError::from)?;

        Ok(())
    }
}
