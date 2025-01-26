use core::ptr;

use super::{error::AcpiError, signature::Signature};

const RSDP_SIGNATURE: Signature<8> = Signature::new_lossy(['R', 'S', 'D', ' ', 'P', 'T', 'R', ' ']);

/// Root System Description Pointer version 1
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(super) struct Rsd1 {
    /// Must contain "RSD PTR ". It stands on a 16-byte boundary & is not null-terminated.
    signature: Signature<8>,
    /// The value to add to all the other bytes (of the Version 1.0 table) to calculate the Checksum of the table.
    checksum: u8,
    // yap
    oem_id: [u8; 6],
    /// The revision of the ACPI. Larger revision numbers are backward compatible to lower revision numbers
    revision: u8,
    /// 32-bit **physical** address of the RSDT table, depracted since version 2.0.
    sdt_addr: u32,
}

/// Root System Description Pointer version 2 or later
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(super) struct RsdX {
    /// See `Rsd1`
    rsd1: Rsd1,
    /// The size of the entire table since offset 0 to the end.
    length: u32,
    /// 64-bit physical address of the XSDT table.
    sdt_addr: u64,
    /// The checksum of the entire table, including both checksum fields.
    extended_checksum: u8,
    _reserved: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(super) enum Rsd {
    V1(Rsd1),
    V2OrLater(RsdX),
}

impl Rsd {
    /// Parse the Root System Descriptor Pointer.
    pub(super) fn parse(rsdp: *const u8) -> Result<Self, AcpiError> {
        // validate rsd pointer
        if unsafe { ptr::read_unaligned(rsdp.cast::<Signature<8>>()) } != RSDP_SIGNATURE {
            return Err(AcpiError::RsdAddress);
        }

        // parse rsdp
        let rsd = unsafe { *(rsdp.cast::<Rsd1>()) };
        Ok(if rsd.revision == 0 {
            Rsd::V1(rsd)
        } else {
            Rsd::V2OrLater(unsafe { *(rsdp.cast::<RsdX>()) })
        })
    }
}

impl Rsd1 {
    pub(super) fn sdt(&self) -> *const u8 {
        self.sdt_addr as *const u8
    }
}

impl RsdX {
    pub(super) fn sdt(&self) -> *const u8 {
        self.sdt_addr as *const u8
    }
}
