use crate::registers::rflags::RFlags;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CpuState {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,

    vector_number: u64,
    error_code: u64,

    iretq_rip: u64,
    iretq_cs: u64,
    iretq_flags: RFlags,
    iretq_rsp: u64,
    iretq_ss: u64,
}
