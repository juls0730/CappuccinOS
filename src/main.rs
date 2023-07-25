#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

extern crate alloc;

mod api;

mod drivers;
mod libs;
mod usr;
mod arch;

use alloc::{format, vec::Vec};
use drivers::{serial, video};
use usr::tty::puts;

#[no_mangle]
pub extern "C" fn _start() -> ! {
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
