use super::idt_set_gate;
use crate::libs::util::hcf;
use crate::{log_error, log_info};

#[no_mangle]
pub extern "C" fn exception_handler(int: u64, eip: u64, cs: u64, eflags: u64) -> ! {
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
        0x0F => {
            log_error!("IDE");
        }
        0xFF => {
            log_error!("EXCEPTION!");
        }
        _ => {
            log_error!("EXCEPTION!");
        }
    }
    log_info!(
        "INT: {:x} EIP: {:X}, CS: {:X}, EFLAGS: {:b}",
        int,
        eip,
        cs,
        eflags
    );

    hcf();
}

#[naked]
pub extern "C" fn div_error() {
    unsafe {
        core::arch::asm!(
            // WHY DOESN'T PUSH DO THIS CORRECTLY
            "mov rdi, 0x00",
            "call exception_handler",
            "add esp, 4",
            "iretq",
            options(noreturn)
        );
    }
}

#[naked]
pub extern "C" fn invalid_opcode() {
    unsafe {
        core::arch::asm!(
            "mov rdi, 0x06",
            "call exception_handler",
            "add esp, 4",
            "iretq",
            options(noreturn)
        );
    }
}

#[naked]
pub extern "C" fn double_fault() {
    unsafe {
        core::arch::asm!(
            "mov rdi, 0x08",
            "call exception_handler",
            "add esp, 4",
            "iretq",
            options(noreturn)
        );
    }
}

#[naked]
pub extern "C" fn general_protection_fault() {
    unsafe {
        core::arch::asm!(
            "mov rdi, 0x0D",
            "call exception_handler",
            "add esp, 4",
            "iretq",
            options(noreturn)
        );
    }
}

#[naked]
pub extern "C" fn page_fault() {
    unsafe {
        core::arch::asm!(
            "mov rdi, 0x0E",
            "call exception_handler",
            "add esp, 4",
            "iretq",
            options(noreturn)
        );
    }
}

#[naked]
pub extern "C" fn generic_handler() {
    unsafe {
        core::arch::asm!(
            "mov rdi, 0xFF",
            "call exception_handler",
            "add esp, 4",
            "iretq",
            options(noreturn)
        );
    }
}

pub fn set_exceptions() {
    idt_set_gate(0x00, div_error as u64, 0x28, 0xEE);
    idt_set_gate(0x06, invalid_opcode as u64, 0x28, 0xEE);
    idt_set_gate(0x08, double_fault as u64, 0x28, 0xEE);
    idt_set_gate(0x0D, general_protection_fault as u64, 0x28, 0xEE);
    idt_set_gate(0x0E, page_fault as u64, 0x28, 0xEE);
}
