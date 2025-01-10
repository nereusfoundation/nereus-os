use core::arch::asm;

use descriptor::SegmentDescriptor;
use mem::VirtualAddress;

mod descriptor;

static GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

/// GDT Descriptor with size of table and pointer to the table (paging applies).
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct GdtDescriptor {
    /// Size of GDT in bytes subtracted by 1. This subtraction occurs because the maximum value of Size is 65535, while the GDT can be up to 65536 bytes in length (8192 entries). Further, no GDT can have a size of 0 bytes.
    size: u16,
    /// The linear address of the GDT (not the physical address, paging applies).
    offset: VirtualAddress,
}

/// The Global Descriptor Table contains entries telling the CPU about memory segments and their
/// permissions.
#[allow(dead_code)]
#[repr(align(0x1000))]
#[derive(Debug)]
pub(super) struct GlobalDescriptorTable {
    null: SegmentDescriptor,
    kernel_code: SegmentDescriptor,
    kernel_data: SegmentDescriptor,
    user_code: SegmentDescriptor,
    user_data: SegmentDescriptor,
    // todo: add TSS for user mode and double-faults
}

impl GlobalDescriptorTable {
    /// Initialize a new GDT
    const fn new() -> Self {
        GlobalDescriptorTable {
            null: SegmentDescriptor::null(),
            kernel_code: SegmentDescriptor::kernel_code(),
            kernel_data: SegmentDescriptor::kernel_data(),
            user_code: SegmentDescriptor::user_code(),
            user_data: SegmentDescriptor::user_data(),
        }
    }
}

/// Load the GlobalDescriptorTable
///
/// # Safety
/// Caller must guarantee that the GDT is valid.
#[inline]
pub(super) unsafe fn load() {
    let gdt_ptr = &GDT as *const GlobalDescriptorTable;

    let desc = GdtDescriptor {
        size: (size_of::<GlobalDescriptorTable>() - 1) as u16,
        offset: gdt_ptr as u64,
    };

    // load gdt
    unsafe {
        asm!("lgdt [{}]", in(reg) &desc as *const GdtDescriptor, options(readonly, nostack, preserves_flags))
    }

    // reload segment registers
    unsafe {
        asm!(
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            // Return Far (pops CS and IP)
            "push 0x08",
            "lea {tmp}, [2f + rip]",
            "push {tmp}",
            "retfq",
            "2:",
            tmp = out(reg) _,

            options(preserves_flags)
        );
    }
}
