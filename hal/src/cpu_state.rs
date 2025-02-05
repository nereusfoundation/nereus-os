use crate::registers::rflags::RFlags;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CpuState {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    pub vector_number: u64,
    pub error_code: u64,

    pub iretq_rip: u64,
    pub iretq_cs: u64,
    pub iretq_flags: RFlags,
    pub iretq_rsp: u64,
    pub iretq_ss: u64,
}

impl CpuState {
    /// Creates a new CPU context.
    pub fn new(
        iretq_ss: u64,
        iretq_rsp: u64,
        iretq_flags: RFlags,
        iretq_cs: u64,
        iretq_rip: u64,
        rbp: u64,
    ) -> CpuState {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rbp,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
            vector_number: 0,
            error_code: 0,
            iretq_rip,
            iretq_cs,
            iretq_flags,
            iretq_rsp,
            iretq_ss,
        }
    }
}
