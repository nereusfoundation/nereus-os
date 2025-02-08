use core::{
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use mem::{PhysicalAddress, PAGE_SIZE};

use crate::vmm::{error::VmmError, object::VmFlags, AllocationType, VMM};

use super::ApicError;

const SPURIOUS_INTERRUPT_VECTOR_OFFSET: usize = 0xF0;
const EOI_OFFSET: usize = 0xB0;
const TASK_PRIORITY_OFFSET: usize = 0x80;
const LOCAL_APIC_ID_OFFSET: usize = 0x20;

pub(in crate::io) const LVT_TIMER_OFFSET: usize = 0x320;
pub(in crate::io) const INITIAL_COUNT_OFFSET: usize = 0x380;
pub(in crate::io) const CURRENT_COUNT_OFFSET: usize = 0x390;
pub(in crate::io) const DIVIDE_CONFIGURATION_OFFSET: usize = 0x3E0;

static LAPIC: AtomicPtr<u8> = AtomicPtr::new(ptr::null_mut::<u8>());

/// Attempts to retrive the address of the LAPIC registers.
pub(crate) fn get() -> Result<*mut u8, ApicError> {
    let load = LAPIC.load(Ordering::Relaxed);
    if load.is_null() {
        Err(ApicError::LapicUninitialized)
    } else {
        Ok(load)
    }
}

pub(crate) fn initialize(base: PhysicalAddress) -> Result<(), VmmError> {
    let mut locked = VMM.locked();
    let vmm = locked.get_mut().ok_or(VmmError::VmmUnitialized)?;
    // lapic registers are at a boundary of 4KB
    let lapic_registers = vmm.alloc(
        PAGE_SIZE,
        VmFlags::WRITE | VmFlags::MMIO | VmFlags::NO_CACHE,
        AllocationType::Address(base),
    )?;

    // todo: model the registers as a struct
    unsafe {
        // more info: https://wiki.osdev.org/APIC#Local_APIC_configuration
        let spurious_vector_register = lapic_registers
            .add(SPURIOUS_INTERRUPT_VECTOR_OFFSET)
            .cast::<u32>();

        // spurious vector value of 0xFF and enable apic software
        spurious_vector_register.write_volatile(0xFF | (1 << 8));

        let task_priority_register = lapic_registers.add(TASK_PRIORITY_OFFSET).cast::<u32>();

        // set priority to 0 so no interrupts are blocked
        task_priority_register.write_volatile(0x0);
    }

    LAPIC.store(lapic_registers.as_ptr(), Ordering::Relaxed);

    Ok(())
}

/// Send the lapic the signal that an interrupt has been handled.
pub(crate) fn eoi() -> Result<(), ApicError> {
    let lapic_address = get()?;
    unsafe {
        // mmio to register has already been mapped in enable function.
        let eoi_register = lapic_address.add(EOI_OFFSET) as *mut u32;
        // signal end of interrupt
        eoi_register.write_volatile(0);
    }

    Ok(())
}

/// Returns the ID of the local apic.
pub(super) fn lapic_id() -> Result<u8, ApicError> {
    let lapic_address = get()?;
    unsafe {
        let id_reigster = lapic_address.add(LOCAL_APIC_ID_OFFSET);
        Ok(*id_reigster)
    }
}
