// Shitty keyboard driver

use crate::{arch::interrupts, libs::io::inb};

struct ModStatuses {
	escape: bool,
	tab: bool,
	win: bool,
	ctrl: bool,
	alt: bool,
	shift: bool,
	caps: bool,
	num_lock: bool,
	scr_lock: bool,
}

pub struct Key<'a> {
	pub mod_key: bool,
	pub printable: bool,
	pub pressed: bool,
	pub key: &'a str
}

static mut EXTENDED_KEY: bool = false;

static mut MOD_STATUSES: ModStatuses = ModStatuses {
	escape: false,
	tab: false,
	win: false,
	ctrl: false,
	alt: false,
	shift: false,
	caps: false,
	num_lock: false,
	scr_lock: false,
};

pub fn init_keyboard() {
	interrupts::idt_set_gate(interrupts::InterruptIndex::Keyboard.as_u8(), keyboard_interrupt_handler, 0x28, 0xEE);
	crate::libs::logging::log_ok("Keyboard initialized\n");
}

extern "x86-interrupt" fn keyboard_interrupt_handler() {
	unsafe {
		interrupts::PICS.notify_end_of_interrupt(interrupts::InterruptIndex::Keyboard.as_u8());
	}

	let scancode = inb(0x60);

	let key = parse_key(scancode);

	if let Some(key) = key {
		crate::usr::tty::handle_key(key);
	}
}

fn parse_key(scancode: u8) -> Option<Key<'static>> {
	// If the key was unpressed then return, see guard statements
	// if (scancode & 128) == 128 {
	// 	return;
	// }
	match scancode {
		0x01 => {
			unsafe {
				MOD_STATUSES.escape = true;
			}
			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: true,
				key: "Esc"
			});
		},
		0x02 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "1"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "!"
				});
			}
		},
		0x03 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "2"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "@"
				});
			}
		},
		0x04 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "3"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "#"
				});
			}
		},
		0x05 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "4"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "$"
				});
			}
		},
		0x06 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "5"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "%"
				});
			}
		},
		0x07 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "6"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "^"
				});
			}
		},
		0x08 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "7"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "&"
				});
			}
		},
		0x09 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "8"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "*"
				});
			}
		},
		0x0A => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "9"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "("
				});
			}
		},
		0x0B => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "0"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: ")"
				});
			}
		},
		0x0C => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "-"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "_"
				});
			}
		},
		0x0D => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "="
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "+"
				});
			}
		},
		0x0E => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "Backspace"
			});
		},
		0x0F => {
			unsafe {
				MOD_STATUSES.tab = true;
			}
			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: true,
				key: "Tab"
			});
		},
		0x10 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "a"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "A"
				});
			}
		},
		0x11 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "w"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "W"
				});
			}
		},
		0x12 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "e"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "E"
				});
			}
		},
		0x13 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "r"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "R"
				});
			}
		},
		0x14 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "t"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "T"
				});
			}
		},
		0x15 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "y"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "Y"
				});
			}
		},
		0x16 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "u"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "U"
				});
			}
		},
		0x17 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "i"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "I"
				});
			}
		},
		0x18 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "o"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "O"
				});
			}
		},
		0x19 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "p"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "P"
				});
			}
		},
		0x1A => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "["
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "{"
				});
			}
		},
		0x1B => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "]"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "}"
				});
			}
		},
		0x1C => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "Enter"
			});
		},
		0x1D => {
			unsafe {
				MOD_STATUSES.ctrl = true;
			}

			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "RCtrl"
				});
			}

			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: true,
				key: "LCtrl"
			});
		},
		0x1E => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "a"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "A"
				});
			}
		},
		0x1F => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "s"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "S"
				});
			}
		},
		0x20 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "d"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "D"
				});
			}
		},
		0x21 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "f"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "F"
				});
			}
		},
		0x22 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "g"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "G"
				});
			}
		},
		0x23 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "h"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "H"
				});
			}
		},
		0x24 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "j"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "J"
				});
			}
		},
		0x25 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "k"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "K"
				});
			}
		},
		0x26 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "l"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "L"
				});
			}
		},
		0x27 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: ";"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: ":"
				});
			}
		},
		0x28 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "'"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "\""
				});
			}
		},
		0x29 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "`"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "~"
				});
			}
		},
		0x2A => {
			unsafe {
				MOD_STATUSES.shift = true;
			}
			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: true,
				key: "LShift"
			})
		},
		0x2B => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "\\"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "|"
				});
			}
		},
		0x2C => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "z"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "Z"
				});
			}
		},
		0x2D => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "x"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "X"
				});
			}
		},
		0x2E => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "c"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "C"
				});
			}
		},
		0x2F => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "v"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "V"
				});
			}
		},
		0x30 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "b"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "B"
				});
			}
		},
		0x31 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "n"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "N"
				});
			}
		},
		0x32 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "m"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "M"
				});
			}
		},
		0x33 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: ","
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "<"
				});
			}
		},
		0x34 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "."
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: ">"
				});
			}
		},
		0x35 => {
			if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "/"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "?"
				});
			}
		},
		0x36 => {
			unsafe {
				MOD_STATUSES.shift = true;
			}
			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: true,
				key: "RShift"
			});
		},
		0x37 => {
			return Some(Key {
				mod_key: false,
				printable: true,
				pressed: true,
				key: "*"
			});
		},
		0x38 => {
			unsafe {
				MOD_STATUSES.alt = true;
			}
			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: true,
				key: "Alt"
			});
		},
		0x39 => {
			return Some(Key {
				mod_key: false,
				printable: true,
				pressed: true,
				key: " "
			});
		},
		0x3A => {
			unsafe {
				MOD_STATUSES.caps = !MOD_STATUSES.caps;
			}
			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: true,
				key: "CapsLock"
			});
		},
		0x3B => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F1"
			});
		},
		0x3C => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F2"
			});
		},
		0x3D => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F3"
			});
		},
		0x3E => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F4"
			});
		},
		0x3F => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F5"
			});
		},
		0x40 => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F6"
			});
		},
		0x41 => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F7"
			});
		},
		0x42 => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F8"
			});
		},
		0x43 => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F9"
			});
		},
		0x44 => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F10"
			});
		},
		0x45 => {
			unsafe {
				MOD_STATUSES.num_lock = !MOD_STATUSES.num_lock;
			}

			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: true,
				key: "NumLock"
			});
		},
		0x46 => {
			unsafe {
				MOD_STATUSES.scr_lock = !MOD_STATUSES.scr_lock;
			}

			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: true,
				key: "ScrLock"
			});
		},
		0x47 => {
			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "Home"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "7"
				});
			}
		},
		0x48 => {
			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "CurUp"
				});
			}

			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "CurUp"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "8"
				});
			}
		},
		0x49 => {
			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "PgUp"
				});
			}

			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "PgUp"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "9"
				});
			}
		},
		0x4A => {
			return Some(Key {
				mod_key: false,
				printable: true,
				pressed: true,
				key: "-"
			});
		},
		0x4B => {
			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "CurLeft"
				});
			}

			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "CurLeft"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "4"
				});
			}
		},
		0x4C => {
			if unsafe { MOD_STATUSES.num_lock } == false {
				return None;
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "5"
				});
			}
		},
		0x4D => {
			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "CurRight"
				});
			}

			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "CurRight"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "6"
				});
			}
		},
		0x4E => {
			return Some(Key {
				mod_key: false,
				printable: true,
				pressed: true,
				key: "+"
			});
		},
		0x4F => {
			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "End"
				});
			}
			
			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "End"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "1"
				});
			}
		},
		0x50 => {
			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "CurDown"
				});
			}

			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "CurDown"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "2"
				});
			}
		},
		0x51 => {
			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "PgDn"
				});
			}

			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "PgDn"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "3"
				});
			}
		},
		0x52 => {
			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "Insert"
				});
			}

			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "Insert"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "0"
				});
			}
		},
		0x53 => {
			if unsafe { EXTENDED_KEY } == true {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "Del"
				});
			}

			if unsafe { MOD_STATUSES.num_lock } == false {
				return Some(Key {
					mod_key: false,
					printable: false,
					pressed: true,
					key: "Del"
				});
			} else {
				return Some(Key {
					mod_key: false,
					printable: true,
					pressed: true,
					key: "0"
				});
			}
		},
		0x57 => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F11"
			});
		},
		0x58 => {
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: true,
				key: "F10"
			});
		},
		0xAA => {
			unsafe {
				MOD_STATUSES.shift = false;
			}
			return Some(Key {
				mod_key: true,
				printable: false,
				pressed: false,
				key: "LShift"
			});
		},
		0xE0 => {
			unsafe {
				EXTENDED_KEY = !EXTENDED_KEY;
			}
			return Some(Key {
				mod_key: false,
				printable: false,
				pressed: false,
				key: "E0"
			})
		}
		_ => None
	}
}