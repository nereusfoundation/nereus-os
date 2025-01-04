use core::arch::asm;

use crate::instructions::cpuid::Cpuid;

#[derive(Clone, Copy)]
pub struct Msr(Cpuid);

impl Msr {
    /// Returns `Some(Msr)` if CPU supports model specific registers (CPUID.01h:EDX[bit 5]).
    pub fn new(cpuid: Cpuid) -> Option<Msr> {
        let available = unsafe { cpuid.get(0x1) }.edx & (1 << 5) != 0;
        available.then_some(Msr(cpuid))
    }

    pub fn get_cpuid(self) -> Cpuid {
        self.0
    }

    /// Reads a 64-bit Model-Specific Register (MSR) at the given `index`.
    ///
    /// # Safety
    /// Caller must specify a valid index and be in privilege level 0.
    pub unsafe fn read(self, index: u32) -> u64 {
        let (high, low): (u32, u32);
        unsafe {
            asm!(
                "rdmsr",
                in("ecx") index,
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
    pub unsafe fn write(self, index: u32, val: u64) {
        let low = val as u32;
        let high = (val >> 32) as u32;

        unsafe {
            asm!(
                "wrmsr",
                in("ecx") index,
                in("eax") low, in("edx") high,
                options(nostack, preserves_flags),
            );
        }
    }
}
