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
        // bit 9 is reserved
        /// Places the local APIC in the x2APIC mode.
        const X2APIC_ENABLE = 1 << 10;
        /// Enables or disables the local Apic
        const LAPIC_ENABLE = 1 << 11;
        /// Addres sof APIC Base registers
        const APIC_BASE = 0xFFFFFFFFFF << 12;
    }
}
#[derive(Debug, thiserror_no_std::Error)]
pub enum ApicError {
    #[error("APIC_BASE register was used, but is not available on this CPU")]
    ApicUnavailable,
    #[error("The X2APIC feature was used, but is not available on this CPU")]
    X2ApicUnavailable,
}

// Safety: IA32_EFER is a valid MSR index.
unsafe impl ModelSpecificRegister for Apic {
    const MSR_INDEX: u32 = IA32_APIC_BASE;
    type ReadError = ApicError;
    type WriteError = ApicError;

    /// Write the IA32_APIC_BASE register if feature is available to CPU. Returns an error value on failure.
    ///
    /// # Safety
    /// Caller must be in privilege level 0. The caller must also guarnatee that the base address field is valid (if provided).
    unsafe fn write(self, msr: Msr) -> Result<(), ApicError> {
        if Self::available(msr.get_cpuid()) {
            if self.contains(Apic::X2APIC_ENABLE) && !Self::x2apic_available(msr.get_cpuid()) {
                return Err(ApicError::X2ApicUnavailable);
            }

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

    /// Check whether the X2APIC feauture is available to the CPU
    pub fn x2apic_available(cpuid: Cpuid) -> bool {
        unsafe { cpuid.get(0x80000001) }.ecx & (1 << 21) != 0
    }

    /// Extracts physical base address of apic registers.
    pub fn base(&self) -> u64 {
        self.bits() & 0x_000F_FFFF_FFFF_F000
    }
}
