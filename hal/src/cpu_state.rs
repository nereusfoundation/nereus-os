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
