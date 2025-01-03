pub mod efer;
pub mod msr_guard;

use bitflags::Flags;
use msr_guard::Msr;

/// An MSR register.
///
/// # Safety
/// MSR_INDEX must be a valid index.
pub unsafe trait ModelSpecificRegister: Sized + Flags<Bits = u64> {
    const MSR_INDEX: u32;
    type ReadError;
    type WriteError;

    /// Read a specific register if MSR feature is available to CPU. Returns an error value on
    /// failure.
    ///
    /// # Safety
    /// Caller must be in privilege level 0.
    unsafe fn read(msr: Msr) -> Result<Self, Self::ReadError> {
        Ok(Self::from_bits_truncate(unsafe {
            msr.read(Self::MSR_INDEX)
        }))
    }

    /// Write a specific register if MSR feature is available to CPU. Returns an error value on failure.
    ///
    /// # Safety
    /// Caller must be in privilege level 0.
    unsafe fn write(self, msr: Msr) -> Result<(), Self::WriteError> {
        unsafe { msr.write(Self::MSR_INDEX, self.bits()) };
        Ok(())
    }
}
