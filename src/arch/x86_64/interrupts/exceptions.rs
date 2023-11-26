use super::idt_set_gate;
use crate::libs::util::hcf;
use crate::{log_error, log_info};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Registers {
    // Pushed by wrapper
    int: usize,

    // Pushed by push_gprs in crate::arch::x86_64
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rbp: usize,
    rdi: usize,
    rsi: usize,
    rdx: usize,
    rcx: usize,
    rbx: usize,
    rax: usize,

    // Pushed by interrupt
    rip: usize,
    cs: usize,
    rflags: usize,
    rsp: usize,
    ss: usize,
}

extern "C" fn exception_handler(registers: u64) {
    crate::println!("{:X?}", registers);

    let registers = unsafe { *(registers as *const Registers) };

    crate::println!("{:X?}", registers);

    let int = registers.int;

    match int {
        0x00 => {
            log_error!("DIVISION ERROR!");
        }
        0x06 => {
            log_error!("INVALID OPCODE!");
        }
        0x08 => {
            log_error!("DOUBLE FAULT!");
        }
        0x0D => {
            log_error!("GENERAL PROTECTION FAULT!");
        }
        0x0E => {
            log_error!("PAGE FAULT!");
        }
        0xFF => {
            log_error!("EXCEPTION!");
        }
        _ => {
            log_error!("EXCEPTION!");
        }
    }

    log_info!(
        "INT: {:x} RIP: {:X}, CS: {:X}, EFLAGS: {:b}",
        int,
        registers.rip,
        registers.cs,
        registers.rflags
    );

    crate::arch::stack_trace::print_stack_trace(6, registers.rbp as u64);
}

// *macro intensifies*
macro_rules! exception_function {
    ($code:expr, $handler:ident, $recoverable:literal) => {
        #[inline(always)]
        extern "C" fn $handler() {
            crate::arch::push_gprs();

            unsafe {
                core::arch::asm!(
                    "push {0:r}",
                    "mov rdi, rsp",
                    "call {1}",
                    "pop {0:r}",
                    "mov rsp, rdi",
                    in(reg) $code,
                    sym exception_handler,
                );
            };

            if $recoverable {
                crate::println!("TODO: Recover gracefully ;~;");
                hcf();
            } else {
                hcf();
            }
        }
    };
}

exception_function!(0x00, div_error, true);
exception_function!(0x06, invalid_opcode, true);
exception_function!(0x08, double_fault, false);
exception_function!(0x0D, general_protection_fault, true);
// TODO: fix the page fault then gracefully return.
exception_function!(0x0E, page_fault, false);
exception_function!(0xFF, generic_handler, true);

pub fn set_exceptions() {
    for i in 0..32 {
        idt_set_gate(i, generic_handler as u64);
    }

    idt_set_gate(0x00, div_error as u64);
    idt_set_gate(0x06, invalid_opcode as u64);
    idt_set_gate(0x08, double_fault as u64);
    idt_set_gate(0x0D, general_protection_fault as u64);
    idt_set_gate(0x0E, page_fault as u64);
}
