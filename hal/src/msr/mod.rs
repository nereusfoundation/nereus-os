pub mod efer;

use bitflags::Flags;

extern "C" {
    pub fn cpu_has_msr() -> bool;
    pub fn get_msr(index: u32) -> u64;

    pub fn set_msr(index: u32, value: u64);
}

pub trait ModelSpecificRegister: Sized + Flags<Bits = u64> {
    const MSR_INDEX: u32;

    /// Read a specific register if MSR feature is available to CPU.
    fn read() -> Option<Self> {
        if unsafe { cpu_has_msr() } {
            Some(Self::from_bits_truncate(unsafe {
                get_msr(Self::MSR_INDEX)
            }))
        } else {
            None
        }
    }

    /// Write a specific register if MSR feature is available to CPU. Returns whether it is available.
    fn write(self) -> bool {
        if unsafe { cpu_has_msr() } {
            unsafe { set_msr(Self::MSR_INDEX, self.bits()) }
            true
        } else {
            false
        }
    }
}
