use hal::{
    instructions::cpuid::Cpuid,
    registers::msr::{self, apic::Apic, msr_guard::Msr, ModelSpecificRegister},
};

#[derive(Debug, thiserror_no_std::Error)]
pub(crate) enum ApicError {
    #[error("The CPUID feature is unavailable to the CPU")]
    CpuidUnavailable,
    #[error("The Model-Specific-Register feature is unavailable to the CPU")]
    MsrUnavailable,
    #[error("Error while using IA32_APIC_BASE register: {0}")]
    Msr(#[from] msr::apic::ApicError),
}

/// Initializes the Advanced Programmable Interrupt Controller.
pub(crate) fn initialize() -> Result<(), ApicError> {
    let cpuid = Cpuid::new().ok_or(ApicError::CpuidUnavailable)?;
    let msr = Msr::new(cpuid).ok_or(ApicError::MsrUnavailable)?;
    let apic = unsafe { Apic::read(msr)? };
    let _address = apic.base();
    Ok(())
}
