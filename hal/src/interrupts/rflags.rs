use core::arch::asm;

use bitflags::bitflags;

bitflags! {
    /// Wrapper of the RFLAGS register
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub(crate) struct RFlags : u64 {
        const CARRY = 1 << 0;
        // bit 1 reserved and always set to 1
        const RESERVED_1 = 1 << 1;
        const PARITY = 1 << 2;
        // bit 3 reserved
        const AUXILIARY_CARRY = 1 << 4;
        // bit 5 reserved
        const ZERO = 1 << 6;
        const SIGN = 1 << 7;
        const TRAP = 1 << 8;
        const INTERRUPTS_ENABLED = 1 << 9;
        const DIRECTION = 1 << 10;
        const OVERFLOW = 1 << 11;
        const IO_PRIVILEGE_LEVEL = 0b11 << 12;
        const NESTED_TASK = 1 << 14;
        // bit 15 reserved
        const RESUME = 1 << 16;
        const VIRTUAL_8086_MODE = 1 << 17;
        const ACCESS_CONTROL = 1 << 18;
        const VIRTUAL_INTERRUPT = 1 << 19;
        const VIRTUAL_INTERRUPT_PENDING = 1 << 20;
        const ID = 1 << 21;
        // 22 - 63 are reserved
    }
}

impl RFlags {
    /// Retrieve the RFLAGS register using the `pushfq` instruction
    pub fn read() -> RFlags {
        let rflags: u64;
        unsafe {
            asm!(
            "pushfq",
            "pop {0}",
            out(reg) rflags,
            );
        }
        RFlags::from_bits_truncate(rflags)
    }
}
