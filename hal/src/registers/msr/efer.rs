use crate::instructions::cpuid::Cpuid;

use super::{ModelSpecificRegister, Msr};
use bitflags::bitflags;

const IA32_EFER: u32 = 0xC000_0080;

bitflags! {
    /// Extended Feature Enable Register
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct Efer: u64 {
        /// System call extensions
        const SCE = 1 << 0;
        // bits  1-7 reserved
        /// Long mode enable (indicated long mode can be used but is not necessarily active)
        const LME = 1 << 8;
        // bit 9 reserved
        /// Long mode active (indicates long mode is active)
        const LMA = 1 << 10;
        /// No-Execute Enable (activates feature that allows to mark pages as NX)
        const NXE = 1 << 11;
        /// Secure Virtual Machine Enable
        const SVME = 1 << 12;
        /// Secure Virtual Machine Enable
        const LMSLE = 1 << 13;
        /// Fast FXSAVE/FXRSTOR
        const FFXSR = 1 << 14;
        /// Translation Cache Extension
        const TCE = 1 << 15;
        // bits 16-63 reserved
    }
}

#[derive(Debug, thiserror_no_std::Error)]
pub enum EferError {
    #[error("NXE was specified, but is not available on this CPU")]
    NXEUnavailable,
}

// Safety: IA32_EFER is a valid MSR index.
unsafe impl ModelSpecificRegister for Efer {
    const MSR_INDEX: u32 = IA32_EFER;
    type ReadError = ();
    type WriteError = EferError;

    unsafe fn write(self, msr: Msr) -> Result<(), EferError> {
        if !self.contains(Efer::NXE) || Self::nx_available(msr.get_cpuid()) {
            // Safety: Caller guarantees that we are in privilege level 0, Self::MSR_INDEX is a valid index.
            unsafe { msr.write(Self::MSR_INDEX, self.bits()) }
            Ok(())
        } else {
            Err(EferError::NXEUnavailable)
        }
    }
}
impl Efer {
    /// Check whether the NX feature is available to the CPU
    pub fn nx_available(cpuid: Cpuid) -> bool {
        unsafe { cpuid.get(0x80000001) }.edx & (1 << 20) != 0
    }

    /// Write `self` to MSR.
    ///
    /// # Safety
    /// This must be called in privilege level 0.
    /// If `self` contains `Efer::NXE`, NXE must be available on this CPU.
    pub unsafe fn write_unchecked(self, msr: Msr) {
        unsafe { msr.write(Self::MSR_INDEX, self.bits()) }
    }
}
