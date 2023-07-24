#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

extern crate alloc;

mod api;

mod drivers;
mod libs;
mod usr;

pub mod arch;

use alloc::{format, vec::Vec};
use drivers::{serial, video};
use usr::tty::puts;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial::init_serial();

    let message = b"Hello world from the kernel.";

    for (_i, &byte) in message.iter().enumerate() {
        serial::write_serial(byte);
    }

    arch::interrupts::idt_init();

    unsafe {
        arch::interrupts::PICS.initialize();
    }

    usr::shell::init_shell();

    loop {
        // Interrupts still work, and this will prevent 100% CPU usage on one core.
        // I am unaware of the consequences of this and am unsure if this will stay permanently.
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
