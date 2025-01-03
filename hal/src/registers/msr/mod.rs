pub mod efer;

use core::{
    arch::{asm, x86_64::__cpuid},
    ops::BitXor,
};

use bitflags::Flags;

use super::rflags::RFlags;

fn cpuid_available() -> bool {
    let before = RFlags::read();
    unsafe {
        RFlags::write(before.bitxor(RFlags::ID));
    }
    let after = RFlags::read();

    // restore old bitflags
    unsafe {
        RFlags::write(before);
    }

    after != before
}

/// Check whether or not a CPU supports model specific registers (CPUID.01h:EDX[bit 5]). May fail
/// if cpuid instruction is not available
fn cpu_has_msr() -> Result<bool, MsrError> {
    if cpuid_available() {
        // EAX=1: Processor Info and Feature Bits, bit 5 = MSR support
        Ok(unsafe { __cpuid(0x1).edx & (1 << 5) != 0 })
    } else {
        Err(MsrError::NoCpuid)
    }
}

pub trait ModelSpecificRegister: Sized + Flags<Bits = u64> {
    const MSR_INDEX: u32;

    /// Read a specific register if MSR feature is available to CPU. Returns an error value on
    /// failure.
    fn read() -> Result<Self, MsrError> {
        if cpu_has_msr()? {
            Ok(Self::from_bits_truncate(unsafe { Self::read_raw() }))
        } else {
            Err(MsrError::MsrFeatureMissing)
        }
    }

    /// Write a specific register if MSR feature is available to CPU. Returns an error value on failure.
    fn write(self) -> Result<(), MsrError> {
        if cpu_has_msr()? {
            unsafe { Self::write_raw(self.bits()) }
            Ok(())
        } else {
            Err(MsrError::MsrFeatureMissing)
        }
    }

    /// Retrieve a 64-bit model specific register
    ///
    /// # Safety
    /// Caller must specify a valid index and be in privilege level 0.
    unsafe fn read_raw() -> u64 {
        let (high, low): (u32, u32);
        unsafe {
            asm!(
                "rdmsr",
                in("ecx") Self::MSR_INDEX,
                out("eax") low, out("edx") high,
                options(nomem, nostack, preserves_flags),
            );
        }
        ((high as u64) << 32) | (low as u64)
    }

    /// Write a 64-bit model specific register
    ///
    /// # Safety
    /// Caller must specify a valid register value and be in privilege level 0.
    #[inline]
    unsafe fn write_raw(val: u64) {
        let low = val as u32;
        let high = (val >> 32) as u32;

        unsafe {
            asm!(
                "wrmsr",
                in("ecx") Self::MSR_INDEX,
                in("eax") low, in("edx") high,
                options(nostack, preserves_flags),
            );
        }
    }
}

#[derive(Debug, thiserror_no_std::Error)]
pub enum MsrError {
    #[error("CPUID instruction unavailable")]
    NoCpuid,
    #[error("MSR CPU Feature not available")]
    MsrFeatureMissing,
}
