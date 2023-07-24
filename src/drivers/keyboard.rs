// Shitty keyboard driver

use crate::{arch::interrupts, libs::io::inb};

#[derive(Clone, Copy)]
pub struct ModStatuses {
    pub escape: bool,
    pub tab: bool,
    pub win: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub caps: bool,
    pub num_lock: bool,
    pub scr_lock: bool,
}

pub struct Key<'a> {
    pub mod_key: bool,
    pub printable: bool,
    pub pressed: bool,
    pub name: &'a str,
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

pub fn init_keyboard(function_ptr: fn(key: Key, mods: ModStatuses)) {
    unsafe { FUNCTION_PTR = function_ptr }

    interrupts::idt_set_gate(
        interrupts::InterruptIndex::Keyboard.as_u8(),
        keyboard_interrupt_handler,
        0x28,
        0xEE,
    );
    crate::libs::logging::log_ok("Keyboard initialized\n");
}

static mut FUNCTION_PTR: fn(key: Key, mods: ModStatuses) = dummy;

fn dummy(_key: Key, _mods: ModStatuses) {}

extern "x86-interrupt" fn keyboard_interrupt_handler() {
    unsafe {
        interrupts::PICS.notify_end_of_interrupt(interrupts::InterruptIndex::Keyboard.as_u8());
    }

    let scancode = inb(0x60);

    let key = parse_key(scancode);

    if let Some(key) = key {
        unsafe { FUNCTION_PTR(key, MOD_STATUSES) }
    }
}

fn parse_key(scancode: u8) -> Option<Key<'static>> {
    match scancode {
        0x01 => {
            unsafe {
                MOD_STATUSES.escape = true;
            }
            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: true,
                name: "Esc",
            });
        }
        0x02 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "1",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "!",
                });
            }
        }
        0x03 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "2",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "@",
                });
            }
        }
        0x04 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "3",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "#",
                });
            }
        }
        0x05 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "4",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "$",
                });
            }
        }
        0x06 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "5",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "%",
                });
            }
        }
        0x07 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "6",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "^",
                });
            }
        }
        0x08 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "7",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "&",
                });
            }
        }
        0x09 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "8",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "*",
                });
            }
        }
        0x0A => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "9",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "(",
                });
            }
        }
        0x0B => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "0",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: ")",
                });
            }
        }
        0x0C => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "-",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "_",
                });
            }
        }
        0x0D => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "=",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "+",
                });
            }
        }
        0x0E => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "Backspace",
            });
        }
        0x0F => {
            unsafe {
                MOD_STATUSES.tab = true;
            }
            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: true,
                name: "Tab",
            });
        }
        0x10 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "a",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "A",
                });
            }
        }
        0x11 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "w",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "W",
                });
            }
        }
        0x12 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "e",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "E",
                });
            }
        }
        0x13 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "r",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "R",
                });
            }
        }
        0x14 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "t",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "T",
                });
            }
        }
        0x15 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "y",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "Y",
                });
            }
        }
        0x16 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "u",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "U",
                });
            }
        }
        0x17 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "i",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "I",
                });
            }
        }
        0x18 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "o",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "O",
                });
            }
        }
        0x19 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "p",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "P",
                });
            }
        }
        0x1A => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "[",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "{",
                });
            }
        }
        0x1B => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "]",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "}",
                });
            }
        }
        0x1C => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "Enter",
            });
        }
        0x1D => {
            unsafe {
                MOD_STATUSES.ctrl = true;
            }

            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "RCtrl",
                });
            }

            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: true,
                name: "LCtrl",
            });
        }
        0x1E => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "a",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "A",
                });
            }
        }
        0x1F => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "s",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "S",
                });
            }
        }
        0x20 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "d",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "D",
                });
            }
        }
        0x21 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "f",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "F",
                });
            }
        }
        0x22 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "g",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "G",
                });
            }
        }
        0x23 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "h",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "H",
                });
            }
        }
        0x24 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "j",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "J",
                });
            }
        }
        0x25 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "k",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "K",
                });
            }
        }
        0x26 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "l",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "L",
                });
            }
        }
        0x27 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: ";",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: ":",
                });
            }
        }
        0x28 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "'",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "\"",
                });
            }
        }
        0x29 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "`",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "~",
                });
            }
        }
        0x2A => {
            unsafe {
                MOD_STATUSES.shift = true;
            }
            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: true,
                name: "LShift",
            });
        }
        0x2B => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "\\",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "|",
                });
            }
        }
        0x2C => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "z",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "Z",
                });
            }
        }
        0x2D => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "x",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "X",
                });
            }
        }
        0x2E => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "c",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "C",
                });
            }
        }
        0x2F => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "v",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "V",
                });
            }
        }
        0x30 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "b",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "B",
                });
            }
        }
        0x31 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "n",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "N",
                });
            }
        }
        0x32 => {
            if unsafe { MOD_STATUSES.shift ^ MOD_STATUSES.caps } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "m",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "M",
                });
            }
        }
        0x33 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: ",",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "<",
                });
            }
        }
        0x34 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: ".",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: ">",
                });
            }
        }
        0x35 => {
            if unsafe { MOD_STATUSES.shift } == false {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "/",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "?",
                });
            }
        }
        0x36 => {
            unsafe {
                MOD_STATUSES.shift = true;
            }
            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: true,
                name: "RShift",
            });
        }
        0x37 => {
            return Some(Key {
                mod_key: false,
                printable: true,
                pressed: true,
                name: "*",
            });
        }
        0x38 => {
            unsafe {
                MOD_STATUSES.alt = true;
            }
            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: true,
                name: "Alt",
            });
        }
        0x39 => {
            return Some(Key {
                mod_key: false,
                printable: true,
                pressed: true,
                name: " ",
            });
        }
        0x3A => {
            unsafe {
                MOD_STATUSES.caps = !MOD_STATUSES.caps;
            }
            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: true,
                name: "CapsLock",
            });
        }
        0x3B => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F1",
            });
        }
        0x3C => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F2",
            });
        }
        0x3D => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F3",
            });
        }
        0x3E => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F4",
            });
        }
        0x3F => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F5",
            });
        }
        0x40 => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F6",
            });
        }
        0x41 => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F7",
            });
        }
        0x42 => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F8",
            });
        }
        0x43 => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F9",
            });
        }
        0x44 => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F10",
            });
        }
        0x45 => {
            unsafe {
                MOD_STATUSES.num_lock = !MOD_STATUSES.num_lock;
            }

            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: true,
                name: "NumLock",
            });
        }
        0x46 => {
            unsafe {
                MOD_STATUSES.scr_lock = !MOD_STATUSES.scr_lock;
            }

            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: true,
                name: "ScrLock",
            });
        }
        0x47 => {
            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "Home",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "7",
                });
            }
        }
        0x48 => {
            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "CurUp",
                });
            }

            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "CurUp",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "8",
                });
            }
        }
        0x49 => {
            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "PgUp",
                });
            }

            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "PgUp",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "9",
                });
            }
        }
        0x4A => {
            return Some(Key {
                mod_key: false,
                printable: true,
                pressed: true,
                name: "-",
            });
        }
        0x4B => {
            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "CurLeft",
                });
            }

            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "CurLeft",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "4",
                });
            }
        }
        0x4C => {
            if unsafe { MOD_STATUSES.num_lock } == false {
                return None;
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "5",
                });
            }
        }
        0x4D => {
            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "CurRight",
                });
            }

            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "CurRight",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "6",
                });
            }
        }
        0x4E => {
            return Some(Key {
                mod_key: false,
                printable: true,
                pressed: true,
                name: "+",
            });
        }
        0x4F => {
            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "End",
                });
            }

            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "End",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "1",
                });
            }
        }
        0x50 => {
            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "CurDown",
                });
            }

            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "CurDown",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "2",
                });
            }
        }
        0x51 => {
            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "PgDn",
                });
            }

            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "PgDn",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "3",
                });
            }
        }
        0x52 => {
            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "Insert",
                });
            }

            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "Insert",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "0",
                });
            }
        }
        0x53 => {
            if unsafe { EXTENDED_KEY } == true {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "Del",
                });
            }

            if unsafe { MOD_STATUSES.num_lock } == false {
                return Some(Key {
                    mod_key: false,
                    printable: false,
                    pressed: true,
                    name: "Del",
                });
            } else {
                return Some(Key {
                    mod_key: false,
                    printable: true,
                    pressed: true,
                    name: "0",
                });
            }
        }
        0x57 => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F11",
            });
        }
        0x58 => {
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: true,
                name: "F10",
            });
        }
        0xAA => {
            unsafe {
                MOD_STATUSES.shift = false;
            }
            return Some(Key {
                mod_key: true,
                printable: false,
                pressed: false,
                name: "LShift",
            });
        }
        0xE0 => {
            unsafe {
                EXTENDED_KEY = !EXTENDED_KEY;
            }
            return Some(Key {
                mod_key: false,
                printable: false,
                pressed: false,
                name: "E0",
            });
        }
        _ => None,
    }
}
