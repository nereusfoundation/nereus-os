use core::arch::naked_asm;

use framebuffer::color;

use crate::{assign_isr, println};

use super::{descriptor::GateType, InterruptDescriptorTable};
use hal::cpu_state::CpuState;

assign_isr!(
    0, GateType::TrapGate
    3, GateType::TrapGate);

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

// Interrupt Service Routines

/// Declare an Inetrrupt Service Routine
#[macro_export]
macro_rules! declare_isr {
    ($isr_number:expr) => {
        paste::paste! {
           #[repr(align(16))]
           #[naked]
           extern "C" fn [<isr_stub_ $isr_number>] (){
               unsafe {
                   ::core::arch::naked_asm!(
                       // push dummy error code
                       "push 0",
                       // push vector number
                       "push {isr_number}",
                       "jmp {interrupt_stub}",
                       isr_number = const $isr_number,
                       interrupt_stub = sym interrupt_stub,
                   );
               }
           }
        }
    };
    (error $isr_number:expr) => {
        paste::paste! {
           #[repr(align(16))]
           #[naked]
           extern "C" fn [<isr_stub_ $isr_number>] (){
               unsafe {
                   ::core::arch::naked_asm!(
                       // push vector number
                       "push {isr_number}",
                       "jmp {interrupt_stub}",
                       isr_number = const $isr_number,
                       interrupt_stub = sym interrupt_stub,
                   );
               }
           }
        }
    };
}

/// Fills IDT with the provided iSRs. Must only be called once.
#[macro_export]
macro_rules! assign_isr {
    ($($isr_number:expr, $gate_type:expr $(, $error:ident)?)*) => {
        paste::paste! {

            $(
                $crate::declare_isr!($($error)? $isr_number);
            )*

            impl InterruptDescriptorTable {
                pub(super) fn assign_handlers(&mut self) {
                    $(
                        self.set_handler(
                            $isr_number,
                            [<isr_stub_ $isr_number>] as usize as u64,
                            0,
                            0,
                            $gate_type,
                        );
                    )*
                }
            }
        }
    };
}
