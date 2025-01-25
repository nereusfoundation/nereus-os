use core::ptr::null;

/// Size of stack that is used when an interrupt occurres and the cpu is not in ring0.
const KERNEL_INTERRUPT_STACK_SIZE: usize = 0x5000;
/// 0x9: 64-bit TSS (Available) [System Segment Access Byte](https://wiki.osdev.org/Global_Descriptor_Table)
pub(super) const TSS_AVAILABLE_FLAGS: u8 = 0x9;

// note: this does not have to be marked as mutable, because the loader maps every kernel elf page
// with the same flags.
/// Stack used for double faults.
static IST_STACK: [u8; KERNEL_INTERRUPT_STACK_SIZE] = [0; KERNEL_INTERRUPT_STACK_SIZE];

/// Stack used for mode switches.
static RSP_STACK: [u8; KERNEL_INTERRUPT_STACK_SIZE] = [0; KERNEL_INTERRUPT_STACK_SIZE];

pub(super) static mut TSS: TaskStateSegment = TaskStateSegment::new();

#[repr(C, packed(4))]
#[derive(Debug, Copy, Clone)]
pub(super) struct TaskStateSegment {
    _reserved0: u32,
    /// The first stack pointer used to load the stack when a privilege level change occurs from a lower privilege level to a higher one.
    rsp0: *const u8,
    _rsp1: u64,
    _rsp2: u64,
    _reserved1: u64,
    /// Interrupt Stack Table. The Stack Pointers used to load the stack when an entry in the Interrupt Descriptor Table has an IST value other than 0.
    ///
    /// Note: The hardware implementation starts indexing with 1, thus the 0th index of this table
    /// must be described as 1 in the IDT entry entry.
    ist: [*const u8; 7],
    _reserved_2: u64,
    _reserved_3: u16,
    /// I/O Map Base Address Field. Contains a 16-bit offset from the base of the TSS to the I/O Permission Bit Map.
    iopb: u16,
}

/// Safety: The TSS is never mutated.
unsafe impl Send for TaskStateSegment {}
/// Safety: The TSS is never mutated.
unsafe impl Sync for TaskStateSegment {}

impl TaskStateSegment {
    /// Creates a new task state segment and with one static stack for double faults and one for
    /// mode switched.
    const fn new() -> Self {
        let rsp0 = unsafe { RSP_STACK.as_ptr().add(KERNEL_INTERRUPT_STACK_SIZE) };

        let ist0 = unsafe { IST_STACK.as_ptr().add(KERNEL_INTERRUPT_STACK_SIZE) };

        Self {
            // effectively disable IO map => no longer used in modern systems.
            iopb: size_of::<TaskStateSegment>() as u16,
            _rsp1: 0,
            rsp0,
            _rsp2: 0,
            ist: {
                let mut ist = [null(); 7];
                ist[0] = ist0;
                ist
            },
            _reserved0: 0,
            _reserved1: 0,
            _reserved_2: 0,
            _reserved_3: 0,
        }
    }
}
