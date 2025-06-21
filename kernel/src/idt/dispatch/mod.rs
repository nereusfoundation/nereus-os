use core::arch::{asm, naked_asm};

use error::{ErrorCode, PageFaultErrorCode};
use framebuffer::color;
use hal::{cpu_state::CpuState, hlt_loop};
use scheduler::Scheduler;

use crate::{
    drivers::keyboard::KEYBOARD,
    io::{apic::lapic, inb},
    loginfo, pit, println, scheduling, serial_println,
};

mod error;
pub(super) mod handler;
pub(super) mod macros;

fn dispatch(state: &CpuState) -> &CpuState {
    let vector_number = state.vector_number;
    let error_code = state.error_code;

    match vector_number {
        0 => {
            println!(color::ERROR, " [ERROR]: division by 0 EXCEPTION");
            hlt_loop();
        }
        3 => {
            loginfo!("breakpoint EXCEPTION");
        }
        14 => {
            println!(
                color::ERROR,
                " [ERROR]: page FAULT, error code: {:?}",
                PageFaultErrorCode::from_bits_truncate(error_code as u32)
            );
            serial_println!(
                " [ERROR]: page FAULT, error code: {:?}",
                PageFaultErrorCode::from_bits_truncate(error_code as u32)
            );
            let cr2: u64;

            unsafe {
                asm!("mov {}, cr2", out(reg) cr2, options(nostack, nomem, preserves_flags));
            }

            println!(color::ERROR, " [INFO ]: faulting address: {:#x}", cr2);
            serial_println!(" [INFO]: faulting address: {:#x}", cr2);
            hlt_loop();
        }
        32 => {
            // SAFETY: hardware interrupts are disabled until after the handler is called.
            lapic::eoi()
                .expect("LAPIC must have been initialized before enabling hardware interrupts!");
            return <scheduling::PerCoreScheduler as Scheduler>::run(state);
        }

        33 => {
            let scancode = unsafe { inb(0x60) };
            let mut binding = KEYBOARD.lock();
            binding.handle(scancode);
            lapic::eoi()
                .expect("LAPIC must have been initialized before enabling hardware interrupts!");
        }
        34 => {
            pit::tick();
            lapic::eoi()
                .expect("LAPIC must have been initialized before enabling hardware interrupts!");
        }

        unknown => {
            println!(
                color::ERROR,
                " [ERROR]: unknown EXCEPTION: {:#x}, error code (if applicable): {:?}",
                unknown,
                ErrorCode::from_bits_truncate(error_code as u32)
            );
            hlt_loop();
        }
    }

    state
}

#[unsafe(naked)]
extern "C" fn interrupt_stub() {
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
