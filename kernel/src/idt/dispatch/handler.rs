use crate::{assign_isr, idt::descriptor::GateType};

// Interrupt Service Routines
assign_isr!(
    0, GateType::TrapGate, 0
    1, GateType::TrapGate, 0
    2, GateType::InterruptGate, 0
    3, GateType::TrapGate, 0
    4, GateType::TrapGate, 0
    5, GateType::TrapGate, 0
    6, GateType::TrapGate, 0
    7, GateType::TrapGate, 0
    8, GateType::TrapGate, 1, error
    9, GateType::TrapGate, 0
    10, GateType::TrapGate, 0, error
    11, GateType::TrapGate, 0, error
    12, GateType::TrapGate, 0, error
    13, GateType::TrapGate, 0, error
    14, GateType::TrapGate, 0, error
    15, GateType::TrapGate, 0
    16, GateType::TrapGate, 0
    17, GateType::TrapGate, 0, error
    18, GateType::TrapGate, 0
    19, GateType::TrapGate, 0
    20, GateType::TrapGate, 0
    21, GateType::TrapGate, 0, error
    28, GateType::TrapGate, 0
    29, GateType::TrapGate, 0, error
    30, GateType::TrapGate, 0, error
    0x20, GateType::InterruptGate, 0 // pit interrupts
    0x21, GateType::InterruptGate, 0 // keyboard interrupts
);
