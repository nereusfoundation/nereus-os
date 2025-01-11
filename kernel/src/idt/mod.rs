use core::{arch::asm, cell::LazyCell};

use descriptor::{GateDescriptor, GateFlags, GateType};
use mem::VirtualAddress;
use sync::spin::SpinLock;

use crate::gdt::KERNEL_CS;

mod descriptor;
mod handler;

const IDT_MAX_DESCRIPTORS: usize = 256;

static IDTR: SpinLock<LazyCell<IdtDescriptor>> = SpinLock::new(LazyCell::new(IdtDescriptor::new));

static IDT: SpinLock<LazyCell<InterruptDescriptorTable>> = SpinLock::new(LazyCell::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.assign_handlers();
    idt
}));

/// IDT Descriptor with size of table and pointer to the table (paging applies).
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct IdtDescriptor {
    /// Index of the last entry in the IDT
    size: u16,
    /// The linear address of the IDT (not the physical address, paging applies).
    offset: VirtualAddress,
}

impl IdtDescriptor {
    fn new() -> IdtDescriptor {
        let idt = IDT.lock();

        IdtDescriptor {
            size: (size_of::<GateDescriptor>() * IDT_MAX_DESCRIPTORS) as u16 - 1, // todo: change size to right one
            offset: LazyCell::force(&idt) as *const InterruptDescriptorTable as u64,
        }
    }
}

#[repr(align(16))]
#[derive(Debug)]
struct InterruptDescriptorTable([GateDescriptor; IDT_MAX_DESCRIPTORS]);

impl InterruptDescriptorTable {
    const fn new() -> Self {
        Self([GateDescriptor::null(); IDT_MAX_DESCRIPTORS])
    }

    const fn set_handler(
        &mut self,
        vector: usize,
        handler_address: u64,
        ist: u8,
        dpl: u8,
        gate_type: GateType,
    ) {
        self.0[vector] = GateDescriptor::new(
            handler_address,
            KERNEL_CS,
            ist,
            GateFlags::new(gate_type, dpl, true),
        );
    }
}

pub(super) unsafe fn load() {
    let idtr = IDTR.lock();
    // load idt
    unsafe {
        asm!("lidt [{}]", in(reg) LazyCell::force(&idtr), options(readonly, nostack, preserves_flags))
    }
}
