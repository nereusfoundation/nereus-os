use mem::{PhysicalAddress, VirtualAddress, PAGE_SIZE};
use sync::locked::Locked;

use crate::vmm::{error::VmmError, object::VmFlags, AllocationType, VMM};

use super::ApicError;

const SPURIOUS_INTERRUPT_VECTOR_OFFSET: usize = 0xF0;
const EOI_OFFSET: usize = 0xB0;
const TASK_PRIORITY_OFFSET: usize = 0x80;
const LOCAL_APIC_ID_OFFSET: usize = 0x20;

static LAPIC: Locked<VirtualAddress> = Locked::new();

pub(crate) fn initialize(base: PhysicalAddress) -> Result<(), VmmError> {
    let mut locked = VMM.locked();
    let vmm = locked.get_mut().ok_or(VmmError::VmmUnitialized)?;
    // lapic registers are at a boundary of 4KB
    let virtual_base = vmm.alloc(
        PAGE_SIZE,
        VmFlags::WRITE | VmFlags::MMIO | VmFlags::NO_CACHE,
        AllocationType::Address(base),
    )?;

    // todo: model the registers as a struct
    unsafe {
        // more info: https://wiki.osdev.org/APIC#Local_APIC_configuration
        let lapic_registers = virtual_base as *const u8;
        let spurious_vector_register =
            lapic_registers.add(SPURIOUS_INTERRUPT_VECTOR_OFFSET) as *mut u32;

        // spurious vector value of 0xFF and enable apic software
        spurious_vector_register.write_volatile(0xFF | (1 << 8));

        let task_priority_register = lapic_registers.add(TASK_PRIORITY_OFFSET) as *mut u32;

        // set priority to 0 so no interrupts are blocked
        task_priority_register.write_volatile(0x0);
    }

    LAPIC.initialize(virtual_base);

    Ok(())
}

/// Send the lapic the signal that an interrupt has been handled.
pub(crate) fn eoi() -> Result<(), ApicError> {
    let locked = LAPIC.locked();
    let lapic_address = *locked.get().ok_or(ApicError::LapicUninitialized)?;
    unsafe {
        // mmio to register has already been mapped in enable function.
        let eoi_register = (lapic_address as *mut u8).add(EOI_OFFSET) as *mut u32;
        // signal end of interrupt
        eoi_register.write_volatile(0);
    }

    Ok(())
}

/// Returns the ID of the local apic.
pub(super) fn lapic_id() -> Result<u8, ApicError> {
    let locked = LAPIC.locked();
    let lapic_address = *locked.get().ok_or(ApicError::LapicUninitialized)?;
    unsafe {
        let id_reigster = (lapic_address as *const u8).add(LOCAL_APIC_ID_OFFSET);
        Ok(*id_reigster)
    }
}
