/// Entry of the Interrupt Descriptor Table.
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(super) struct GateDescriptor {
    /// Lower 16 bits of the ISR's address
    offset_low: u16,
    /// GDT segment selector that the CPU will load into CS before calling the ISR
    segment_selector: u16,
    /// IST in the TSS that the CPU will load into RSP
    ist: u8,
    /// type + 0 + dpl + present
    flags: GateFlags,
    /// The higher 16 bits of the lower 32 bits of the ISR's address
    offset_middle: u16,
    /// The higher 32 bits of the ISR's address
    offset_high: u32,
    // set to 0
    _reserved: u32,
}

impl GateDescriptor {
    pub(super) const fn new(offset: u64, segment_selector: u16, ist: u8, flags: GateFlags) -> Self {
        let offset_low = (offset & 0xFFFF) as u16;
        let offset_middle = ((offset >> 16) & 0xFFFF) as u16;
        let offset_high = ((offset >> 32) & 0xFFFFFFFF) as u32;

        Self {
            offset_low,
            segment_selector,
            ist,
            flags,
            offset_middle,
            offset_high,
            _reserved: 0,
        }
    }

    pub(super) const fn null() -> Self {
        Self::new(0, 0, 0, GateFlags::new(GateType::TrapGate, 0, false))
    }
}

/// Flags of a [`GateDescriptor`]
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default)]
pub(super) struct GateFlags(u8);

impl GateFlags {
    pub(super) const fn new(r#type: GateType, dpl: u8, present: bool) -> Self {
        GateFlags(r#type.bits() | (dpl << 6) | ((present as u8) << 7))
    }
}

/// Types of Idt Descriptors
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub(super) enum GateType {
    /// Clears interrupt flag before calling handler
    InterruptGate = 0,
    /// Does not clear interrupt flag before calling handler. Meaning interrupts can occur, while current one is being handled.
    TrapGate = 1,
}

impl GateType {
    /// Four type bits for GateFlags
    const fn bits(&self) -> u8 {
        match self {
            GateType::InterruptGate => 0b1110,
            GateType::TrapGate => 0b1111,
        }
    }
}
