#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod drivers;
mod libs;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
pub mod arch;

use drivers::{serial, video};

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

	drivers::keyboard::init_keyboard();

	loop {
		// Interrupts still work, and this will prevent 100% CPU usage on one core.
		// I am unaware of the consequences of this and am unsure if this will stay permanently.
		unsafe {
			core::arch::asm!("hlt");
		}
	}
}

#[panic_handler]
fn panic (_info: &core::panic::PanicInfo) -> ! {
	loop {
		unsafe {
			core::arch::asm!("hlt");
		}
	}
}