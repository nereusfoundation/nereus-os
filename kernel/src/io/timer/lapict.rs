use bitflags::bitflags;

use crate::{
    io::apic::ApicError,
    lapic::{
        self, CURRENT_COUNT_OFFSET, DIVIDE_CONFIGURATION_OFFSET, INITIAL_COUNT_OFFSET,
        LVT_TIMER_OFFSET,
    },
};

use super::pit;

/// Initializes the Local APIC Timer and callibrates it using the `crate::io::timer::pit::PIT`.
/// Thus, the LAPIC Timer has the same frequency as the one configured for the PIT.
pub(crate) fn initialize() -> Result<(), ApicError> {
    let div = (DivideConfigurationRegister::BIT0).bits();
    let lapic_address = lapic::get()?;
    // todo: account for CPUID.06H:EAX.ARAT[bit 2] = 1 and CPUID.06H:EAX.ARAT[bit 2] = 0 / CPUID 06H not supported

    // configure the Divide Configuration Register (controls the division factor used to determine the frequency => how often the timer counts down based on the system clock frequency)
    unsafe {
        let div_register = lapic_address.add(DIVIDE_CONFIGURATION_OFFSET).cast::<u32>();
        div_register.write_volatile(div);
    }

    // set apic counter to -1
    unsafe {
        let initial_counter_register = lapic_address.add(INITIAL_COUNT_OFFSET).cast::<u32>();
        initial_counter_register.write_volatile(0xFFFFFFFF);
    }

    // sleep for 10ms
    pit::sleep(10);
    // stop APIC timer
    unsafe {
        let timer_register = lapic_address.add(LVT_TIMER_OFFSET).cast::<u32>();
        timer_register.write_volatile(TimerLocalVectorTableEntry::INTERRUPT_MASK.bits());
    }

    let ticks_in_10ms = 0xFFFFFFFF
        - unsafe {
            lapic_address
                .add(CURRENT_COUNT_OFFSET)
                .cast::<u32>()
                .read_volatile()
        };
    unsafe {
        let timer_register = lapic_address.add(LVT_TIMER_OFFSET).cast::<u32>();
        timer_register
            .write_volatile(TimerLocalVectorTableEntry::periodic(0x20, false, false).bits());
    }

    unsafe {
        let div_register = lapic_address.add(DIVIDE_CONFIGURATION_OFFSET).cast::<u32>();
        div_register.write_volatile(div);
    }
    unsafe {
        let initial_counter_register = lapic_address.add(INITIAL_COUNT_OFFSET).cast::<u32>();
        initial_counter_register.write_volatile(ticks_in_10ms);
    }

    Ok(())
}

bitflags! {
    /// General structure of the LAPIC Timer LVT entries
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    struct TimerLocalVectorTableEntry: u32 {
        /// IDT entry that should be triggered for the specific interrupt.
        const INTERRUPT_VECTOR = 0xFF;
        /// Whether the interrupt has been served or not (0: Idle, 1: Send pending) (read only).
        const DELIVERY_STATUS = 0b1 << 12;
        /// If it is 1 the interrupt is disabled, if 0 is enabled.
        const INTERRUPT_MASK = 0b1 << 16;
        /// Mode to operate in (00b: One-shot, 01b: Periodic, 10b: TSC-Deadline)
        const TIMER_MODE = 0b11 << 17;
    }

}

impl TimerLocalVectorTableEntry {
    /// Creates a new timer LVT entry in the periodic mode.
    const fn periodic(vector: u8, status: bool, mask: bool) -> TimerLocalVectorTableEntry {
        TimerLocalVectorTableEntry::from_bits_truncate(
            (vector as u32) | ((status as u32) << 12) | ((mask as u32) << 16) | ((1u32) << 17),
        )
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    struct DivideConfigurationRegister : u32 {
        const BIT0 = 1;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 3;
    }
}
