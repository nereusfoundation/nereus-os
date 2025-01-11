use core::arch::naked_asm;

use framebuffer::color;
use hal::cpu_state::CpuState;

use crate::println;

pub(super) mod handler;
pub(super) mod macros;

fn dispatch(state: &CpuState) -> &CpuState {
    println!(color::INFO, "Hello DEBUG Interrupt!");
    state
}

#[naked]
extern "C" fn interrupt_stub() {
    unsafe {
        naked_asm!(
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rsi",
            "push rdi",
            "push rbp",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            // pass rsp to the dispatch handler (stack pointer)
            "mov rdi, rsp",
            "call {interrupt_dispatch}",

            // restore the stack pointer returned by the dispatch handler
            "mov rsp, rax",

            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rbp",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            // remove vector number + error code (16 bytes)
            "add rsp, 16",
            "iretq",
            interrupt_dispatch = sym dispatch
        );
    }
}
