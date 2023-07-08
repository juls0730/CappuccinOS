use crate::{arch::interrupts, libs::io::inb};

extern "x86-interrupt" fn keyboard_interrupt_handler() {
	unsafe {
		interrupts::PICS.notify_end_of_interrupt(interrupts::InterruptIndex::Keyboard.as_u8());
	}

	let scancode = inb(0x60);

	// If the key was unpressed then return, see guard statements
	if (scancode & 128) == 128 {
		return;
	}

	let key = match scancode {
		0x02 => Some("1"),
		0x03 => Some("2"),
		0x04 => Some("3"),
		0x05 => Some("4"),
		0x06 => Some("5"),
		0x07 => Some("6"),
		0x08 => Some("7"),
		0x09 => Some("8"),
		0x0a => Some("9"),
		0x0b => Some("0"),
		_ => None
	};

	if let Some(key) = key {
		crate::drivers::video::puts(key);
	}
}

pub fn init_keyboard() {
	interrupts::idt_set_gate(interrupts::InterruptIndex::Keyboard.as_u8(), keyboard_interrupt_handler, 0x28, 0xEE);
	crate::libs::logging::log_ok("Keyboard initialized\n");
}