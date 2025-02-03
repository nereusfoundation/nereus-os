use alloc::vec::Vec;
use error::AcpiError;
use madt::{entry::InterruptSourceOverride, Madt};
use mem::PhysicalAddress;
use rsd::Rsd;
use sdt::Rsdt;
use signature::Signature;

pub(crate) mod error;
pub(crate) mod madt;
pub(crate) mod rsd;
pub(crate) mod sdt;
pub(crate) mod signature;

/// Parses the ACPI Tables.
pub(crate) fn parse(rsdp: *const u8) -> Result<Rsdt, AcpiError> {
    let rsd = Rsd::parse(rsdp)?;
    let sdt = Rsdt::new(rsd)?;
    // todo: parse remaining tables and retrieve system information
    Ok(sdt)
}

/// Parses the MADT and returns the physical address of the local apic registers as well as
/// interrupt source overrides.
pub(crate) fn madt(
    sdt: Rsdt,
) -> Result<(PhysicalAddress, Vec<InterruptSourceOverride>), AcpiError> {
    let madt = unsafe { sdt.parse_table::<Madt>(Signature(*b"APIC"))?.as_ref() };
    Ok((
        madt.lapic_registers(),
        madt.parse_entries::<InterruptSourceOverride>(),
    ))
}
