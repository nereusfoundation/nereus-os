use alloc::vec::Vec;
use bitflags::bitflags;
use mem::{VirtualAddress, PAGE_SIZE};

use crate::{
    acpi::madt::entry::IoApic,
    vmm::{error::VmmError, object::VmFlags, AllocationType, VMM},
};

use super::ApicError;

/// Interrupt Request (IRQ) for PS/2 keyboard entry index
pub(super) const KEYBOARD_IRQ: u8 = 1;
/// Interrupt Request (IRQ) for pit entry index
pub(super) const TIMER_IRQ: u8 = 0;

// I/O APIC Registers for accessing other registers:
/// I/O Register Select: Is used to select the I/O Register to access
const IOREGSEL_OFFSET: usize = 0x00;
/// I/O Window (data): Used to access data selected by IOREGSEL
const IOWIN_OFFSET: usize = 0x10;

// I/O APIC Registers that are accessed using selection registers mentioned above:
/// I/O APIC Redirection tables: The redirection tables: 0x03 - 0x3f with registers starting from 0x10 (read/write)
const IOREDTBL_REGISTERS_OFFSET: u8 = 0x10;

/// Initializes the IO-APICS of the system. Returns the virtual address of the first IO-APIC.
pub(super) fn initialize(io_apics: Vec<IoApic>) -> Result<VirtualAddress, ApicError> {
    let mut locked = VMM.locked();
    let vmm = locked
        .get_mut()
        .ok_or(VmmError::VmmUnitialized)
        .map_err(ApicError::from)?;
    // todo: initialize remaining IO-APICS.
    let io_apic = io_apics.first().ok_or(ApicError::NoIoApic)?;
    let address = io_apic.address();

    let virtual_address = vmm
        .alloc(
            PAGE_SIZE,
            VmFlags::WRITE | VmFlags::MMIO | VmFlags::NO_CACHE,
            AllocationType::Address(address),
        )
        .map_err(ApicError::from)?;

    Ok(virtual_address.as_ptr() as u64)
}

/// Write to the IOAPIC control registers.
///
/// # Safety
/// The caller must ensure that the register specified by the address and offset is valid and can be written to.
unsafe fn write(io_apic_base: u64, offset: u8, value: u32) {
    let reg_select = (io_apic_base + IOREGSEL_OFFSET as u64) as *mut u32;
    let reg_window = (io_apic_base + IOWIN_OFFSET as u64) as *mut u32;

    // write to IOREGSEL to select the register
    reg_select.write_volatile(offset as u32);

    // write to IOWIN to set the new value
    reg_window.write_volatile(value);
}

/// Configure a new redirection entry to handle a hardware interrupt using the specified interrupt handler vector offset.
///
/// # Safety
/// The caller must ensure that the IO APIC address is valid and mapped.
pub(super) unsafe fn configure_redirection_entry(
    io_apic_base: VirtualAddress,
    index: u8,
    idt_vector_index: u8,
    destination_lapic_id: u8,
    enable: bool,
) {
    let low_index = IOREDTBL_REGISTERS_OFFSET + (index * 2);
    let high_index = low_index + 1;

    // construct lower register of redirection entry (delivery mode=000, destination mode=physical, pin polarity=active-high, trigger mode=edge
    let mut lvt = LintLocalVectorTableEntry::from_bits_truncate(idt_vector_index as u32);
    if !enable {
        lvt.insert(LintLocalVectorTableEntry::INTERRUPT_MASK);
    }

    // construct higher register of redirection entry
    let destination = (destination_lapic_id as u32) << 24;

    // write redirection entry
    write(io_apic_base, low_index, lvt.bits());
    write(io_apic_base, high_index, destination);
}

bitflags! {
    /// General structure of the LINT0 and LINT1 LVT entries
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    struct LintLocalVectorTableEntry: u32 {
        /// IDT entry that should be triggered for the specific interrupt.
        const INTERRUPT_VECTOR = 0xFF;
        /// Determines how the APIC should present the interrupt to the processor (000b: Fixed, 010b:
        /// SMI, 100b: NMI, 111b: ExtINT, the other variants are reserved).
        const DELIVERY_MODE = 0b111 << 8;
        /// Whether the interrupt has been served or not (0: Idle, 1: Send pending) (read only).
        const DELIVERY_STATUS = 0b1 << 12;
        /// 0 is active-high, 1 is active-low.
        const PIN_POLARITY = 0b1 << 13;
        /// Used by the APIC for managing level-triggered interrupts (read only).
        const REMOTE_INTERRUPT_REQUEST_REGISTER = 0b1 << 14;
        /// 0 is edge-triggered, 1 is level-triggered.
        const TRIGGER_MODE = 0b1 << 15;
        /// If it is 1 the interrupt is disabled, if 0 is enabled.
        const INTERRUPT_MASK = 0b1 << 16;
    }

}
