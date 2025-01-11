use crate::{assign_isr, idt::descriptor::GateType};

// Interrupt Service Routines
assign_isr!(
    0, GateType::TrapGate
    3, GateType::TrapGate);
