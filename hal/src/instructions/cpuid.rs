use core::{
    arch::x86_64::{CpuidResult, __cpuid},
    ops::BitXor,
};

use crate::registers::rflags::RFlags;

#[derive(Clone, Copy)]
pub struct Cpuid(());

impl Cpuid {
    /// Returns `Some(Cpuid)` if the CPU supports the `cpuid` instruction.
    pub fn new() -> Option<Self> {
        let before = RFlags::read();
        unsafe {
            RFlags::write(before.bitxor(RFlags::ID));
        }
        let after = RFlags::read();

        // restore old bitflags
        unsafe {
            RFlags::write(before);
        }

        (after != before).then_some(Cpuid(()))
    }

    /// Get the result of the `cpuid` instruction for the given `leaf`
    ///
    /// # Safety
    /// `leaf` must be valid for this CPU.
    pub unsafe fn get(self, leaf: u32) -> CpuidResult {
        // Safety: This requires an instance of Cpuid, which means that cpuid is available.
        // The caller guarantees that `leaf` is valid.
        unsafe { __cpuid(leaf) }
    }
}
