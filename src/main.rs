#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![no_std]
#![no_main]

extern crate alloc;

mod api;

mod arch;
mod drivers;
mod libs;
mod sys;
mod usr;

use alloc::format;
use drivers::serial;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    sys::mem::init();

    serial::init_serial();

    arch::interrupts::init();

    usr::shell::init_shell();

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    libs::logging::log_error(&format!("{}", info));
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
