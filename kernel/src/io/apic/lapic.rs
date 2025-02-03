use mem::{PhysicalAddress, VirtualAddress, PAGE_SIZE};
use sync::locked::Locked;

use crate::vmm::{error::VmmError, object::VmFlags, AllocationType, VMM};

const SPURIOUS_INTERRUPT_VECTOR_OFFSET: usize = 0xF0;
const EOI_OFFSET: usize = 0xB0;
const TASK_PRIORITY_OFFSET: usize = 0x80;
const LOCAL_APIC_ID_OFFSET: usize = 0x20;
const LOCAL_APIC_VERSION_OFFSET: usize = 0x30;

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

    LAPIC.initialize(virtual_base);

    Ok(())
}
