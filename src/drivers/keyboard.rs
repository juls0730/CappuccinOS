// Shitty keyboard driver
use crate::arch::{
    interrupts,
    io::{inb, outb},
};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Key<'a> {
    pub mod_key: bool,
    pub pressed: bool,
    pub name: &'a str,
    pub character: Option<char>,
}

static EXTENDED_KEY: AtomicBool = AtomicBool::new(false);

pub extern "x86-interrupt" fn keyboard_interrupt_handler() {
    unsafe {
        interrupts::PICS.notify_end_of_interrupt(interrupts::InterruptIndex::Keyboard.as_u8());
    }

    let scancode = inb(0x60);

    let key = parse_key(scancode);

    if let Some(key) = key {
        crate::usr::shell::handle_key(key)
    }
}

pub fn consume_scancode() {
    let _ = inb(0x60);
}

pub fn set_leds(led_byte: u8) {
    // Command bytes
    outb(0x60, 0xED);
    while !(inb(0x60) == 0xfa) {}
    // Data byte
    outb(0x60, led_byte);
}

fn parse_key(scancode: u8) -> Option<Key<'static>> {
    match scancode {
        0x01 => {
            return Some(Key {
                mod_key: true,
                pressed: true,
                name: "Esc",
                character: None,
            });
        }
        0x02 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "1",
                character: Some('1'),
            });
        }
        0x03 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "2",
                character: Some('2'),
            });
        }
        0x04 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "3",
                character: Some('3'),
            });
        }
        0x05 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "4",
                character: Some('4'),
            });
        }
        0x06 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "5",
                character: Some('5'),
            });
        }
        0x07 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "6",
                character: Some('6'),
            });
        }
        0x08 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "7",
                character: Some('7'),
            });
        }
        0x09 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "8",
                character: Some('8'),
            });
        }
        0x0A => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "9",
                character: Some('9'),
            });
        }
        0x0B => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "0",
                character: Some('0'),
            });
        }
        0x0C => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "-",
                character: Some('-'),
            });
        }
        0x0D => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "=",
                character: Some('='),
            });
        }
        0x0E => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Backspace",
                character: None,
            });
        }
        0x0F => {
            return Some(Key {
                mod_key: true,
                pressed: true,
                name: "Tab",
                character: None,
            });
        }
        0x10 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "q",
                character: Some('q'),
            });
        }
        0x11 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "w",
                character: Some('w'),
            });
        }
        0x12 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "e",
                character: Some('e'),
            });
        }
        0x13 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "r",
                character: Some('r'),
            });
        }
        0x14 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "t",
                character: Some('t'),
            });
        }
        0x15 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "y",
                character: Some('y'),
            });
        }
        0x16 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "u",
                character: Some('u'),
            });
        }
        0x17 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "i",
                character: Some('i'),
            });
        }
        0x18 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "o",
                character: Some('o'),
            });
        }
        0x19 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "p",
                character: Some('p'),
            });
        }
        0x1A => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "[",
                character: Some('['),
            });
        }
        0x1B => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "]",
                character: Some(']'),
            });
        }
        0x1C => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Enter",
                character: None,
            });
        }
        0x1D => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "RCtrl",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: true,
                pressed: true,
                name: "LCtrl",
                character: None,
            });
        }
        0x1E => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "a",
                character: Some('a'),
            });
        }
        0x1F => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "s",
                character: Some('s'),
            });
        }
        0x20 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "d",
                character: Some('d'),
            });
        }
        0x21 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "f",
                character: Some('f'),
            });
        }
        0x22 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "g",
                character: Some('g'),
            });
        }
        0x23 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "h",
                character: Some('h'),
            });
        }
        0x24 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "j",
                character: Some('j'),
            });
        }
        0x25 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "k",
                character: Some('k'),
            });
        }
        0x26 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "l",
                character: Some('l'),
            });
        }
        0x27 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: ";",
                character: Some(';'),
            });
        }
        0x28 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "'",
                character: Some('\''),
            });
        }
        0x29 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "`",
                character: Some('`'),
            });
        }
        0x2A => {
            return Some(Key {
                mod_key: true,
                pressed: true,
                name: "LShift",
                character: None,
            });
        }
        0x2B => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "\\",
                character: Some('\\'),
            });
        }
        0x2C => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "z",
                character: Some('z'),
            });
        }
        0x2D => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "x",
                character: Some('x'),
            });
        }
        0x2E => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "c",
                character: Some('c'),
            });
        }
        0x2F => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "v",
                character: Some('v'),
            });
        }
        0x30 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "b",
                character: Some('b'),
            });
        }
        0x31 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "n",
                character: Some('n'),
            });
        }
        0x32 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "m",
                character: Some('m'),
            });
        }
        0x33 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: ",",
                character: Some(','),
            });
        }
        0x34 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: ".",
                character: Some('.'),
            });
        }
        0x35 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "/",
                character: Some('/'),
            });
        }
        0x36 => {
            return Some(Key {
                mod_key: true,
                pressed: true,
                name: "RShift",
                character: None,
            });
        }
        0x37 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "*",
                character: Some('*'),
            });
        }
        0x38 => {
            return Some(Key {
                mod_key: true,
                pressed: true,
                name: "Alt",
                character: None,
            });
        }
        0x39 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: " ",
                character: Some(' '),
            });
        }
        0x3A => {
            return Some(Key {
                mod_key: true,
                pressed: true,
                name: "CapsLock",
                character: None,
            });
        }
        0x3B => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F1",
                character: None,
            });
        }
        0x3C => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F2",
                character: None,
            });
        }
        0x3D => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F3",
                character: None,
            });
        }
        0x3E => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F4",
                character: None,
            });
        }
        0x3F => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F5",
                character: None,
            });
        }
        0x40 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F6",
                character: None,
            });
        }
        0x41 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F7",
                character: None,
            });
        }
        0x42 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F8",
                character: None,
            });
        }
        0x43 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F9",
                character: None,
            });
        }
        0x44 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F10",
                character: None,
            });
        }
        0x45 => {
            return Some(Key {
                mod_key: true,
                pressed: true,
                name: "NumLock",
                character: None,
            });
        }
        0x46 => {
            return Some(Key {
                mod_key: true,
                pressed: true,
                name: "ScrLock",
                character: None,
            });
        }
        0x47 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "Home",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 7",
                character: Some('7'),
            });
        }
        0x48 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "CurUp",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 8",
                character: Some('8'),
            });
        }
        0x49 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "PgUp",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 9",
                character: Some('9'),
            });
        }
        0x4A => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "-",
                character: Some('-'),
            });
        }
        0x4B => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "CurLeft",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 4",
                character: Some('4'),
            });
        }
        0x4C => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 5",
                character: Some('5'),
            });
        }
        0x4D => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "CurRight",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 6",
                character: Some('6'),
            });
        }
        0x4E => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "+",
                character: Some('+'),
            });
        }
        0x4F => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "End",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 1",
                character: Some('1'),
            });
        }
        0x50 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "CurDown",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 2",
                character: Some('2'),
            });
        }
        0x51 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "PgDn",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 3",
                character: Some('3'),
            });
        }
        0x52 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "Insert",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad 0",
                character: Some('0'),
            });
        }
        0x53 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: true,
                    name: "Del",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "Keypad .",
                character: Some('.'),
            });
        }
        0x57 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F11",
                character: None,
            });
        }
        0x58 => {
            return Some(Key {
                mod_key: false,
                pressed: true,
                name: "F10",
                character: None,
            });
        }
        0x81 => {
            return Some(Key {
                mod_key: true,
                pressed: false,
                name: "Esc",
                character: None,
            });
        }
        0x9D => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    mod_key: false,
                    pressed: false,
                    name: "RCtrl",
                    character: None,
                });
            }

            return Some(Key {
                mod_key: true,
                pressed: false,
                name: "LCtrl",
                character: None,
            });
        }
        0xAA => {
            return Some(Key {
                mod_key: true,
                pressed: false,
                name: "LShift",
                character: None,
            });
        }
        0xE0 => {
            EXTENDED_KEY.store(!EXTENDED_KEY.load(Ordering::SeqCst), Ordering::SeqCst);
            return None;
        }
        _ => None,
    }
}
