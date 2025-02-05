mod ioapic;
pub(crate) mod lapic;

use alloc::vec::Vec;
use hal::{
    instructions::cpuid::Cpuid,
    registers::msr::{self, apic::Apic, msr_guard::Msr, ModelSpecificRegister},
};
use ioapic::{KEYBOARD_IRQ, TIMER_IRQ};
use mem::PhysicalAddress;

use crate::{
    acpi::madt::entry::{InterruptSourceOverride, IoApic},
    vmm,
};

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum ApicError {
    #[error("The CPUID feature is unavailable to the CPU")]
    CpuidUnavailable,
    #[error("The Model-Specific-Register feature is unavailable to the CPU")]
    MsrUnavailable,
    #[error("Error while using IA32_APIC_BASE register: {0}")]
    Msr(#[from] msr::apic::ApicError),
    #[error("{0}")]
    Vmm(#[from] vmm::error::VmmError),
    #[error("LAPIC has not yet been initialzed.")]
    LapicUninitialized,
    #[error("No IOAPIC available.")]
    NoIoApic,
}

/// Checks whether the APIC is present on the machine. Enabling it if it is disabled
pub(crate) fn enable() -> Result<(), ApicError> {
    let cpuid = Cpuid::new().ok_or(ApicError::CpuidUnavailable)?;
    let msr = Msr::new(cpuid).ok_or(ApicError::MsrUnavailable)?;
    let apic = unsafe { Apic::read(msr)? };
    // enable apic if it's disabled
    if !apic.contains(Apic::LAPIC_ENABLE) {
        unsafe {
            apic.union(Apic::LAPIC_ENABLE)
                .write(msr)
                .map_err(ApicError::from)?;
        }
    }

    Ok(())
}

/// Initializes the Advanced Programmable Interrupt Controller.
pub(crate) fn initialize(
    lapic_address: PhysicalAddress,
    overrides: Vec<InterruptSourceOverride>,
    io_apics: Vec<IoApic>,
) -> Result<(), ApicError> {
    lapic::initialize(lapic_address).map_err(ApicError::from)?;
    let io_apic_virtual_address = ioapic::initialize(io_apics)?;

    // configure redirection entires
    let keyboard_source = overrides
        .iter()
        .find(|iso| iso.source() == KEYBOARD_IRQ)
        .map(|iso| iso.gsi() as u8)
        .unwrap_or(KEYBOARD_IRQ);
    unsafe {
        ioapic::configure_redirection_entry(
            io_apic_virtual_address,
            keyboard_source,
            0x21,
            lapic::lapic_id()?,
            true,
        );
    }

    let pit_source = overrides
        .iter()
        .find(|iso| iso.source() == TIMER_IRQ)
        .map(|iso| iso.gsi() as u8)
        .unwrap_or(TIMER_IRQ);
    unsafe {
        ioapic::configure_redirection_entry(
            io_apic_virtual_address,
            pit_source,
            0x22,
            lapic::lapic_id()?,
            true,
        );
    }

    Ok(())
}
