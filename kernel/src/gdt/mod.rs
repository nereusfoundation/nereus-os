use core::{arch::asm, cell::LazyCell};

use descriptor::SegmentDescriptor;
use mem::VirtualAddress;
use sync::spin::SpinLock;
use tss::TSS;

mod descriptor;
mod tss;

pub(crate) const KERNEL_CS: u16 = 0x08;
pub(crate) const KERNEL_DS: u16 = 0x10;
const KERNEL_TSS: u16 = 0x28;

static GDTR: SpinLock<LazyCell<GdtDescriptor>> = SpinLock::new(LazyCell::new(GdtDescriptor::new));
static GDT: SpinLock<LazyCell<GlobalDescriptorTable>> =
    SpinLock::new(LazyCell::new(GlobalDescriptorTable::new));

/// GDT Descriptor with size of table and pointer to the table (paging applies).
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct GdtDescriptor {
    /// Size of GDT in bytes subtracted by 1. This subtraction occurs because the maximum value of Size is 65535, while the GDT can be up to 65536 bytes in length (8192 entries). Further, no GDT can have a size of 0 bytes.
    size: u16,
    /// The linear address of the GDT (not the physical address, paging applies).
    offset: VirtualAddress,
}

impl GdtDescriptor {
    fn new() -> GdtDescriptor {
        let gdt = GDT.lock();
        GdtDescriptor {
            size: (size_of::<GlobalDescriptorTable>() - 1) as u16,
            offset: LazyCell::force(&*gdt) as *const GlobalDescriptorTable as u64,
        }
    }
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
    tss_low: SegmentDescriptor,
    tss_high: SegmentDescriptor,
}

impl GlobalDescriptorTable {
    /// Initialize a new GDT
    fn new() -> Self {
        let (tss_low, tss_high) = unsafe { SegmentDescriptor::tss(&raw const TSS) };

        GlobalDescriptorTable {
            null: SegmentDescriptor::null(),
            kernel_code: SegmentDescriptor::kernel_code(),
            kernel_data: SegmentDescriptor::kernel_data(),
            user_code: SegmentDescriptor::user_code(),
            user_data: SegmentDescriptor::user_data(),
            tss_low,
            tss_high,
        }
    }
}

/// Load the GlobalDescriptorTable
///
/// # Safety
/// Caller must guarantee that the GDT is valid.
#[inline]
pub(super) unsafe fn load() {
    let gdtr = GDTR.lock();

    // load gdt
    unsafe {
        asm!("lgdt [{}]", in(reg) LazyCell::force(&gdtr) as *const GdtDescriptor, options(readonly, nostack, preserves_flags))
    }

    // reload segment registers
    unsafe {
        asm!(
            "mov ax, {ds}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            // Return Far (pops CS and IP)
            "push {cs}",
            "lea {tmp}, [2f + rip]",
            "push {tmp}",
            "retfq",
            "2:",
            tmp = out(reg) _,
            cs = const KERNEL_CS as u64,
            ds = const KERNEL_DS as u64,

            options(preserves_flags)
        );
    }

    // load task switch segment
    unsafe {
        asm!("mov ax, {tss}", "ltr ax", tss = const KERNEL_TSS as u64, options(nostack, preserves_flags))
    }
}
