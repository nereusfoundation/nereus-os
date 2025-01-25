use bitflags::bitflags;

use super::tss::{TaskStateSegment, TSS_AVAILABLE_FLAGS};

/// A reference to a descriptor, which can be loaded into a segment register
/// - Limit: A 20-bit value, tells the maximum addressable unit, either in 1 byte units, or in 4KiB pages. Hence, if you choose page granularity and set the Limit value to 0xFFFFF the segment will span the full 4 GiB address space in 32-bit mode.
/// - Base: A 32-bit value containing the linear address where the segment begins.
///
/// > In 64-bit mode, the Base and Limit values are ignored, each descriptor covers the entire linear address space regardless of what they are set to.
#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Default)]
pub(super) struct SegmentDescriptor {
    /// first 16 bits of the Limit field
    limit_low: u16,
    /// first 16 bits of the Base field
    base_low: u16,
    /// next 8 bits of the Base field
    base_middle: u8,
    access: AccessByte,
    /// last 4 birs of the Limit field and [`SegmentDescriptorFlags`]
    granularity: u8,
    /// last 8 bits of the Base field
    base_high: u8,
}

bitflags! {
    /// Byte for Access Control of a Segment Descriptor
    #[derive(Copy, Clone, Debug, Default)]
    pub(super) struct AccessByte: u8 {
        /// The CPU will set it when the segment is accessed unless set to 1 in advance.
        const ACCESSED              = 1 << 0;
        /// - Code Segments: Readable (write access is never allowed on code segments)
        /// - Data Segments: Writeable (read access is always allowed on data segments)
        const READABLE_WRITEABLE    = 1 << 1;
        /// - Code Selectors: Conforming (If clear (0) code in this segment can only be executed from the ring set in DPL. If set (1) code in this segment can be executed from an equal or lower privilege level.)
        /// - Data Selectors: Direction (If clear (0) the segment grows up. If set (1) the segment grows down, ie. the Offset has to be greater than the Limit.)
        const CONFORMING_DIRECTION  = 1 << 2;
        /// If clear (0) the descriptor defines a data segment. If set (1) it defines a code segment which can be executed from.
        const EXECUTABLE            = 1 << 3;
        /// If clear (0) the descriptor defines a system segment (eg. a Task State Segment). If set (1) it defines a code or data segment.
        const DESCRIPTOR_TYPE       = 1 << 4;
        /// Descriptor privilege level field. Contains the CPU Privilege level of the segment. 0 = highest privilege (kernel), 3 = lowest privilege (user applications).
        const DPL                   = 0b11 << 5;
        /// Allows an entry to refer to a valid segment. Must be set (1) for any valid segment.
        const PRESENT               = 1 << 7;
    }
}

bitflags! {
    /// Flags for Segment Description Configuration
    #[derive(Copy, Clone, Debug, Default)]
    pub(super) struct SegmentDescriptorFlags: u8 {
        /// Long-mode code flag. If set (1), the descriptor defines a 64-bit code segment. When set, DB should always be clear. For any other type of segment (other code types or any data segment), it should be clear (0).
        const LONG_MODE             = 1 << 5;
        /// Size flag. If clear (0), the descriptor defines a 16-bit protected mode segment. If set (1) it defines a 32-bit protected mode segment. A GDT can have both 16-bit and 32-bit selectors at once.
        const SIZE                  = 1 << 6;
        /// Granularity flag, indicates the size the Limit value is scaled by. If clear (0), the Limit is in 1 Byte blocks (byte granularity). If set (1), the Limit is in 4 KiB blocks (page granularity).
        const GRANULARITY           = 1 << 7;
    }
}

impl SegmentDescriptor {
    /// Initialize a new SegmentDescriptor
    const fn new(base: u32, limit: u32, access: AccessByte, flags: SegmentDescriptorFlags) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: ((limit >> 16) & 0x0F) as u8 | (flags.bits() & 0xF0),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }

    /// Kernel Mode Code Segment
    pub(super) const fn kernel_code() -> Self {
        SegmentDescriptor::new(
            0,
            0xFFFFF,
            AccessByte::PRESENT
                .union(AccessByte::DESCRIPTOR_TYPE)
                .union(AccessByte::EXECUTABLE)
                .union(AccessByte::READABLE_WRITEABLE)
                .union(AccessByte::ACCESSED),
            SegmentDescriptorFlags::LONG_MODE.union(SegmentDescriptorFlags::GRANULARITY),
        )
    }

    /// Kernel Mode Data Segment
    pub(super) const fn kernel_data() -> Self {
        SegmentDescriptor::new(
            0,
            0xFFFFF,
            AccessByte::PRESENT
                .union(AccessByte::DESCRIPTOR_TYPE)
                .union(AccessByte::READABLE_WRITEABLE)
                .union(AccessByte::ACCESSED),
            SegmentDescriptorFlags::GRANULARITY.union(SegmentDescriptorFlags::SIZE),
        )
    }

    /// User Mode Code Segment
    pub(super) const fn user_code() -> Self {
        SegmentDescriptor::new(
            0,
            0xFFFFF,
            AccessByte::PRESENT
                .union(AccessByte::DPL)
                .union(AccessByte::DESCRIPTOR_TYPE)
                .union(AccessByte::EXECUTABLE)
                .union(AccessByte::READABLE_WRITEABLE)
                .union(AccessByte::ACCESSED),
            SegmentDescriptorFlags::LONG_MODE.union(SegmentDescriptorFlags::GRANULARITY),
        )
    }

    /// User Mode Data Segment
    pub(super) const fn user_data() -> Self {
        SegmentDescriptor::new(
            0,
            0xFFFFF,
            AccessByte::PRESENT
                .union(AccessByte::DPL)
                .union(AccessByte::DESCRIPTOR_TYPE)
                .union(AccessByte::READABLE_WRITEABLE)
                .union(AccessByte::ACCESSED),
            SegmentDescriptorFlags::GRANULARITY.union(SegmentDescriptorFlags::SIZE),
        )
    }

    pub(super) const fn null() -> Self {
        SegmentDescriptor::new(0, 0, AccessByte::empty(), SegmentDescriptorFlags::empty())
    }

    /// Return the low and high segment descriptors pointing to the specified tss.
    pub(super) fn tss(tss: &'static TaskStateSegment) -> (Self, Self) {
        let tss_address = tss as *const TaskStateSegment as u64;
        let low = SegmentDescriptor::new(
            tss_address as u32,
            (size_of::<TaskStateSegment>() - 1) as u32,
            AccessByte::from_bits_truncate(AccessByte::PRESENT.bits() | TSS_AVAILABLE_FLAGS),
            SegmentDescriptorFlags::empty(),
        );

        let high = SegmentDescriptor::new(
            (tss_address >> 32) as u32,
            0,
            AccessByte::empty(),
            SegmentDescriptorFlags::empty(),
        );

        (low, high)
    }
}
