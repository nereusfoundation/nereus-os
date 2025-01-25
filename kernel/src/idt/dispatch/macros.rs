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
                       interrupt_stub = sym $crate::idt::dispatch::interrupt_stub,
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
                       interrupt_stub = sym $crate::idt::dispatch::interrupt_stub,
                   );
               }
           }
        }
    };
}

/// Fills IDT with the provided iSRs. Must only be called once.
#[macro_export]
macro_rules! assign_isr {
    ($($isr_number:expr, $gate_type:expr, $ist:expr $(, $error:ident)?)*) => {
        paste::paste! {

            $(
                $crate::declare_isr!($($error)? $isr_number);
            )*

            impl $crate::idt::InterruptDescriptorTable {
                pub(in $crate::idt) fn assign_handlers(&mut self) {
                    $(
                        self.set_handler(
                            $isr_number,
                            [<isr_stub_ $isr_number>] as usize as u64,
                            $ist,
                            0,
                            $gate_type,
                        );
                    )*
                }
            }
        }
    };
}
