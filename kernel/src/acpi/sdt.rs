use core::ptr;

use super::{error::AcpiError, signature::Signature, Rsd};

/// Signature of root SystemDescriptorTable for ACPI versions 2.0+
const XSDT_SIGNAUTRE: Signature<4> = Signature(*b"XSDT");
/// Signature of root SystemDescriptorTable for ACPI version 1.0
const RSDT_SIGNAUTRE: Signature<4> = Signature(*b"RSDT");

/// System Descritpro Table Header
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub(super) struct Header {
    signature: Signature<4>,
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

/// Root System Descriptor Table. This table contains pointers to all the other System Description Tables.
#[derive(Copy, Clone, Debug)]
pub(super) struct Rsdt {
    /// Pointer to the root system descriptor table
    ptr: *const Header,
    /// Whether ACPI version 1 or 2+ is being used. Version 1 uses 32-bit pointers.
    version2: bool,
}

impl Rsdt {
    pub(super) fn new(rsd: Rsd) -> Result<Self, AcpiError> {
        let (signature, ptr, version2) = match rsd {
            Rsd::V1(rsd1) => (RSDT_SIGNAUTRE, rsd1.sdt(), false),
            Rsd::V2OrLater(rsdx) => (XSDT_SIGNAUTRE, rsdx.sdt(), true),
        };
        // validate sdt pointer
        if unsafe { ptr::read_unaligned(ptr.cast::<Signature<4>>()) } != signature {
            Err(AcpiError::RsdtAddress)
        } else {
            let casted = ptr.cast::<Header>();
            Ok(Self {
                ptr: casted,
                version2,
            })
        }
    }
}

impl Rsdt {
    /// Parses the given system descritpor table based on it's signature.
    pub(super) fn parse(&self, signature: Signature<4>) -> Result<Header, AcpiError> {
        let header = unsafe { self.ptr.read_unaligned() };
        let ptr_size = if self.version2 { 8 } else { 4 };
        // amount of remaining pointers to the other tables that fit into the total size of the XSDT
        let entries = (header.length as usize - size_of::<Header>()) / ptr_size;
        let base_ptr = unsafe { self.ptr.add(1).cast::<u8>() };
        for i in 0..entries {
            let entry_ptr = unsafe { base_ptr.add(i * ptr_size) };
            let entry = unsafe { **(entry_ptr.cast::<*const Header>()) };
            if signature == entry.signature {
                return Ok(entry);
            }
        }

        Err(AcpiError::TableNotFound(signature))
    }
}
