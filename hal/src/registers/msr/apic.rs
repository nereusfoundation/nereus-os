use bitflags::bitflags;

use crate::instructions::cpuid::Cpuid;

use super::{msr_guard::Msr, ModelSpecificRegister};

const IA32_APIC_BASE: u32 = 0x1B;
bitflags! {
    /// Flags for the Advanced Programmable Interrupt Controler Base Register.
    #[repr(transparent)]
    #[derive( Clone, Copy, Debug)]
    pub struct Apic: u64 {
        // bits 0 - 7 are reserved.
        /// Indicates whether the current processor is the bootstrap processor
        const BSP = 1 << 8;
        // bits 9 - 10 are reserved.
        /// Enables or disables the local Apic
        const LAPIC_ENABLE = 1 << 11;
        /// Specifies the base address of the APIC registers. This 24-bit value is extended by 12 bits at the low end to form the base address.
        const APIC_BASE = 0b111111111111111111111111 << 12;
         // bits 36-63 reserved
    }
}
#[derive(Debug, thiserror_no_std::Error)]
pub enum ApicError {
    #[error("APIC_BASE register was used, but is not available on this CPU")]
    ApicUnavailable,
}

// Safety: IA32_EFER is a valid MSR index.
unsafe impl ModelSpecificRegister for Apic {
    const MSR_INDEX: u32 = IA32_APIC_BASE;
    type ReadError = ApicError;
    type WriteError = ApicError;

    unsafe fn write(self, msr: Msr) -> Result<(), ApicError> {
        if Self::available(msr.get_cpuid()) {
            // Safety: Caller guarantees that we are in privilege level 0, Self::MSR_INDEX is a valid index.
            unsafe { msr.write(Self::MSR_INDEX, self.bits()) }
            Ok(())
        } else {
            Err(ApicError::ApicUnavailable)
        }
    }

    unsafe fn read(msr: Msr) -> Result<Self, ApicError> {
        if Self::available(msr.get_cpuid()) {
            Ok(Self::from_bits_truncate(unsafe {
                msr.read(Self::MSR_INDEX)
            }))
        } else {
            Err(ApicError::ApicUnavailable)
        }
    }
}

impl Apic {
    /// Check whether the IA32_APIC_BASE msr is available to the CPU
    ///
    /// **Warning**: On early AMD K5 processors this is used to indicate support for PGE instead.
    pub fn available(cpuid: Cpuid) -> bool {
        unsafe { cpuid.get(0x80000001) }.edx & (1 << 9) != 0
    }
}
