use error::AcpiError;
use rsd::Rsd;
use sdt::Rsdt;
use signature::Signature;

pub(crate) mod error;
mod rsd;
mod sdt;
pub(crate) mod signature;

/// Parses the ACPI Tables
pub(crate) fn parse(rsdp: *const u8) -> Result<(), AcpiError> {
    let rsd = Rsd::parse(rsdp)?;
    let sdt = Rsdt::new(rsd)?;
    let _madt = sdt.parse(Signature(*b"APIC"))?;
    // todo: parse remaining tables and retrieve system information
    Ok(())
}
