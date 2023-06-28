#![no_std]
#![no_main]

mod drivers;
mod libs;

use drivers::{serial, video};

#[no_mangle]
pub extern "C" fn _start() -> ! {
	serial::init_serial();

	// let garbage_message = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
	let message = b"Hello world from the kernel.";

	for (_i, &byte) in message.iter().enumerate() {
		serial::write_serial(byte);
	}

	let mut rand = libs::rand::Random::new();
	rand.rseed(1234);

	video::init_video();

	loop {
		video::fill_screen(rand.rand() as u32)
	}
}


#[panic_handler]
fn panic (_info: &core::panic::PanicInfo) -> ! {
	loop {}
}