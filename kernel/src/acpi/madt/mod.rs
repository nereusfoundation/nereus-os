use alloc::vec::Vec;
use entry::{LApicAddressOverride, MadtEntry, MadtEntryHeader};
use mem::PhysicalAddress;

use super::sdt::Header;

pub(crate) mod entry;

#[repr(C, packed)]
#[derive(Debug)]
pub(crate) struct Madt {
    header: Header,
    /// Base address of LAPIC registers
    local_apic_address: u32,
    flags: u32,
}

impl Madt {
    /// Returns all Madt entries specified by T in the Madt or an empty vector.
    /// Note: This will panic if called before the global heap allocator has been intialized.
    pub(crate) fn parse_entries<T: Copy + MadtEntry>(&self) -> Vec<T> {
        let mut entries = Vec::default();
        let madt_start = self as *const _ as *const u8;
        let mut pointer = unsafe { madt_start.add(size_of::<Madt>()) };
        // pointer to first byte after madt
        let madt_end = unsafe { madt_start.add(self.header.length() as usize) };
        while pointer < madt_end {
            let entry = unsafe { *(pointer.cast::<MadtEntryHeader>()) };
            // io apic has type 1
            if entry.entry_type == T::ENTRY_TYPE {
                entries.push(unsafe { (pointer.cast::<T>()).read_unaligned() });
            }

            pointer = unsafe { pointer.add(entry.record_length as usize) };
        }

        entries
    }

    /// Retrieves the local apic registers address from the MADT.
    pub(crate) fn lapic_registers(&self) -> PhysicalAddress {
        // check for override or use the one in the header instead.
        self.parse_entries::<LApicAddressOverride>()
            .first()
            .map(|entry| entry.lapic_registers_address)
            .unwrap_or(self.local_apic_address as u64)
    }
}
