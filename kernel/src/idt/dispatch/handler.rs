use crate::{assign_isr, idt::descriptor::GateType};

// Interrupt Service Routines
assign_isr!(
    0, GateType::TrapGate
    1, GateType::TrapGate
    2, GateType::InterruptGate
    3, GateType::TrapGate
    4, GateType::TrapGate
    5, GateType::TrapGate
    6, GateType::TrapGate
    7, GateType::TrapGate
    8, GateType::TrapGate, error
    9, GateType::TrapGate
    10, GateType::TrapGate, error
    11, GateType::TrapGate, error
    12, GateType::TrapGate, error
    13, GateType::TrapGate, error
    14, GateType::TrapGate, error
    15, GateType::TrapGate
    16, GateType::TrapGate
    17, GateType::TrapGate, error
    18, GateType::TrapGate
    19, GateType::TrapGate
    20, GateType::TrapGate
    21, GateType::TrapGate, error
    28, GateType::TrapGate
    29, GateType::TrapGate, error
    30, GateType::TrapGate, error
);
