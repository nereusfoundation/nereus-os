use bitflags::bitflags;

bitflags! {
    /// Error code for page faults. In addition, the value of the CR2 register is set to the virtual address that causes the fault
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub(super) struct PageFaultErrorCode: u32 {
        /// Present: When set, the page fault was caused by a page-protection violation. When not set, it was caused by a non-present page.
        const PRESENT = 1 << 0;
        /// Write: When set, the page fault was caused by a write access. When not set, it was caused by a read access.
        const WRITE = 1 << 1;
        /// User: When set, the page fault was caused while CPL = 3. This does not necessarily mean that the page fault was a privilege violation.
        const USER = 1 << 2;
        /// Reserved Write: When set, one or more page directory entries contain reserved bits which are set to 1. This only applies when the PSE or PAE flags in CR4 are set to 1.
        const RESERVED_WRITE = 1 << 3;
        /// Instruction Fetch: When set, the page fault was caused by an instruction fetch. This only applies when the No-Execute bit is supported and enabled.
        const INSTRUCTION_FETCH = 1 << 4;
        /// Protection Key: When set, the page fault was caused by a protection-key violation. PKRU register (user-mode accesses) or PKRS MSR (supervisor-mode accesses) specifies protection key rights.
        const PROTECTION_KEY = 1 << 5;
        /// Shadow Stack: When set, the page fault was caused by a shadow stack access.
        const SHADOW_STACK = 1 << 6;
        // bits 7 - 14 reserved
        /// Software Guard Extension: When set, the fault was due to an SGX violation. The fault is unrelated to ordinary paging.
        const SGX = 1 << 15;
        // bits 16 - 31 reserved
    }

    /// General purpose error code
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub(super) struct ErrorCode: u32 {
        /// External: If set, means it was a hardware interrupt. Cleared for software interrupts.
        const EXTERNAL = 1 << 0;
        /// IDT: Set if this error code refers to the IDT. If cleared it refers to the GDT or LDT.
        const IDT = 1 << 1;
        /// Table Index: Set if the error code refers to the LDT, cleared if referring to the GDT.
        const TABLE_INDEX = 1 << 2;
        /// Index: The index into the table this error code refers to. This can be seen as a byte offset into the table, much like a GDT selector would.
        const INDEX = 0b11111111111111111111111111111 << 3;
    }

}
