// Shitty keyboard driver
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::arch::{
    interrupts,
    io::{inb, outb},
};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Key<'a> {
    pub pressed: bool,
    pub name: &'a str,
    pub character: Option<char>,
}

static EXTENDED_KEY: AtomicBool = AtomicBool::new(false);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "x86-interrupt" fn keyboard_interrupt_handler() {
    interrupts::PICS
        .lock()
        .write()
        .notify_end_of_interrupt(interrupts::InterruptIndex::Keyboard.as_u8());

    let scancode = inb(0x60);

    let key = parse_key(scancode);

    if let Some(key) = key {
        crate::usr::shell::handle_key(key)
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn consume_scancode() {
    let _ = inb(0x60);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn set_leds(led_byte: u8) {
    // Command bytes
    outb(0x60, 0xED);
    while !(inb(0x60) == 0xfa) {}
    // Data byte
    outb(0x60, led_byte);
}

fn parse_key(mut scancode: u8) -> Option<Key<'static>> {
    if scancode == 0xE0 {
        EXTENDED_KEY.store(!EXTENDED_KEY.load(Ordering::SeqCst), Ordering::SeqCst);
        return None;
    }

    let pressed = scancode & 0x80 == 0x0;
    scancode &= !(1 << 7);

    let key: Option<Key<'static>>;

    match scancode {
        0x01 => {
            key = Some(Key {
                pressed,
                name: "Esc",
                character: Some('\u{001B}'),
            });
        }
        0x02 => {
            return Some(Key {
                pressed,
                name: "1",
                character: Some('1'),
            });
        }
        0x03 => {
            return Some(Key {
                pressed,
                name: "2",
                character: Some('2'),
            });
        }
        0x04 => {
            return Some(Key {
                pressed,
                name: "3",
                character: Some('3'),
            });
        }
        0x05 => {
            return Some(Key {
                pressed,
                name: "4",
                character: Some('4'),
            });
        }
        0x06 => {
            return Some(Key {
                pressed,
                name: "5",
                character: Some('5'),
            });
        }
        0x07 => {
            return Some(Key {
                pressed,
                name: "6",
                character: Some('6'),
            });
        }
        0x08 => {
            return Some(Key {
                pressed,
                name: "7",
                character: Some('7'),
            });
        }
        0x09 => {
            return Some(Key {
                pressed,
                name: "8",
                character: Some('8'),
            });
        }
        0x0A => {
            return Some(Key {
                pressed,
                name: "9",
                character: Some('9'),
            });
        }
        0x0B => {
            return Some(Key {
                pressed,
                name: "0",
                character: Some('0'),
            });
        }
        0x0C => {
            return Some(Key {
                pressed,
                name: "-",
                character: Some('-'),
            });
        }
        0x0D => {
            return Some(Key {
                pressed,
                name: "=",
                character: Some('='),
            });
        }
        0x0E => {
            return Some(Key {
                pressed,
                name: "Backspace",
                character: Some('\u{0008}'),
            });
        }
        0x0F => {
            return Some(Key {
                pressed,
                name: "Tab",
                character: Some('\u{0009}'),
            });
        }
        0x10 => {
            return Some(Key {
                pressed,
                name: "q",
                character: Some('q'),
            });
        }
        0x11 => {
            return Some(Key {
                pressed,
                name: "w",
                character: Some('w'),
            });
        }
        0x12 => {
            return Some(Key {
                pressed,
                name: "e",
                character: Some('e'),
            });
        }
        0x13 => {
            return Some(Key {
                pressed,
                name: "r",
                character: Some('r'),
            });
        }
        0x14 => {
            return Some(Key {
                pressed,
                name: "t",
                character: Some('t'),
            });
        }
        0x15 => {
            return Some(Key {
                pressed,
                name: "y",
                character: Some('y'),
            });
        }
        0x16 => {
            return Some(Key {
                pressed,
                name: "u",
                character: Some('u'),
            });
        }
        0x17 => {
            return Some(Key {
                pressed,
                name: "i",
                character: Some('i'),
            });
        }
        0x18 => {
            return Some(Key {
                pressed,
                name: "o",
                character: Some('o'),
            });
        }
        0x19 => {
            return Some(Key {
                pressed,
                name: "p",
                character: Some('p'),
            });
        }
        0x1A => {
            return Some(Key {
                pressed,
                name: "[",
                character: Some('['),
            });
        }
        0x1B => {
            return Some(Key {
                pressed,
                name: "]",
                character: Some(']'),
            });
        }
        0x1C => {
            return Some(Key {
                pressed,
                name: "Enter",
                character: Some('\u{000A}'),
            });
        }
        0x1D => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "RCtrl",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "LCtrl",
                character: None,
            });
        }
        0x1E => {
            return Some(Key {
                pressed,
                name: "a",
                character: Some('a'),
            });
        }
        0x1F => {
            return Some(Key {
                pressed,
                name: "s",
                character: Some('s'),
            });
        }
        0x20 => {
            return Some(Key {
                pressed,
                name: "d",
                character: Some('d'),
            });
        }
        0x21 => {
            return Some(Key {
                pressed,
                name: "f",
                character: Some('f'),
            });
        }
        0x22 => {
            return Some(Key {
                pressed,
                name: "g",
                character: Some('g'),
            });
        }
        0x23 => {
            return Some(Key {
                pressed,
                name: "h",
                character: Some('h'),
            });
        }
        0x24 => {
            return Some(Key {
                pressed,
                name: "j",
                character: Some('j'),
            });
        }
        0x25 => {
            return Some(Key {
                pressed,
                name: "k",
                character: Some('k'),
            });
        }
        0x26 => {
            return Some(Key {
                pressed,
                name: "l",
                character: Some('l'),
            });
        }
        0x27 => {
            return Some(Key {
                pressed,
                name: ";",
                character: Some(';'),
            });
        }
        0x28 => {
            return Some(Key {
                pressed,
                name: "'",
                character: Some('\''),
            });
        }
        0x29 => {
            return Some(Key {
                pressed,
                name: "`",
                character: Some('`'),
            });
        }
        0x2A => {
            return Some(Key {
                pressed,
                name: "LShift",
                character: None,
            });
        }
        0x2B => {
            return Some(Key {
                pressed,
                name: "\\",
                character: Some('\\'),
            });
        }
        0x2C => {
            return Some(Key {
                pressed,
                name: "z",
                character: Some('z'),
            });
        }
        0x2D => {
            return Some(Key {
                pressed,
                name: "x",
                character: Some('x'),
            });
        }
        0x2E => {
            return Some(Key {
                pressed,
                name: "c",
                character: Some('c'),
            });
        }
        0x2F => {
            return Some(Key {
                pressed,
                name: "v",
                character: Some('v'),
            });
        }
        0x30 => {
            return Some(Key {
                pressed,
                name: "b",
                character: Some('b'),
            });
        }
        0x31 => {
            return Some(Key {
                pressed,
                name: "n",
                character: Some('n'),
            });
        }
        0x32 => {
            return Some(Key {
                pressed,
                name: "m",
                character: Some('m'),
            });
        }
        0x33 => {
            return Some(Key {
                pressed,
                name: ",",
                character: Some(','),
            });
        }
        0x34 => {
            return Some(Key {
                pressed,
                name: ".",
                character: Some('.'),
            });
        }
        0x35 => {
            return Some(Key {
                pressed,
                name: "/",
                character: Some('/'),
            });
        }
        0x36 => {
            return Some(Key {
                pressed,
                name: "RShift",
                character: None,
            });
        }
        0x37 => {
            return Some(Key {
                pressed,
                name: "*",
                character: Some('*'),
            });
        }
        0x38 => {
            return Some(Key {
                pressed,
                name: "Alt",
                character: None,
            });
        }
        0x39 => {
            return Some(Key {
                pressed,
                name: " ",
                character: Some(' '),
            });
        }
        0x3A => {
            return Some(Key {
                pressed,
                name: "CapsLock",
                character: None,
            });
        }
        0x3B => {
            return Some(Key {
                pressed,
                name: "F1",
                character: None,
            });
        }
        0x3C => {
            return Some(Key {
                pressed,
                name: "F2",
                character: None,
            });
        }
        0x3D => {
            return Some(Key {
                pressed,
                name: "F3",
                character: None,
            });
        }
        0x3E => {
            return Some(Key {
                pressed,
                name: "F4",
                character: None,
            });
        }
        0x3F => {
            return Some(Key {
                pressed,
                name: "F5",
                character: None,
            });
        }
        0x40 => {
            return Some(Key {
                pressed,
                name: "F6",
                character: None,
            });
        }
        0x41 => {
            return Some(Key {
                pressed,
                name: "F7",
                character: None,
            });
        }
        0x42 => {
            return Some(Key {
                pressed,
                name: "F8",
                character: None,
            });
        }
        0x43 => {
            return Some(Key {
                pressed,
                name: "F9",
                character: None,
            });
        }
        0x44 => {
            return Some(Key {
                pressed,
                name: "F10",
                character: None,
            });
        }
        0x45 => {
            return Some(Key {
                pressed,
                name: "NumLock",
                character: None,
            });
        }
        0x46 => {
            return Some(Key {
                pressed,
                name: "ScrLock",
                character: None,
            });
        }
        0x47 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "Home",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad 7",
                character: Some('7'),
            });
        }
        0x48 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "CurUp",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad 8",
                character: Some('8'),
            });
        }
        0x49 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "PgUp",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad 9",
                character: Some('9'),
            });
        }
        0x4A => {
            return Some(Key {
                pressed,
                name: "-",
                character: Some('-'),
            });
        }
        0x4B => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "CurLeft",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad 4",
                character: Some('4'),
            });
        }
        0x4C => {
            return Some(Key {
                pressed,
                name: "Keypad 5",
                character: Some('5'),
            });
        }
        0x4D => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "CurRight",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad 6",
                character: Some('6'),
            });
        }
        0x4E => {
            return Some(Key {
                pressed,
                name: "+",
                character: Some('+'),
            });
        }
        0x4F => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "End",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad 1",
                character: Some('1'),
            });
        }
        0x50 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "CurDown",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad 2",
                character: Some('2'),
            });
        }
        0x51 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "PgDn",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad 3",
                character: Some('3'),
            });
        }
        0x52 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "Insert",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad 0",
                character: Some('0'),
            });
        }
        0x53 => {
            if EXTENDED_KEY.load(Ordering::SeqCst) == true {
                return Some(Key {
                    pressed,
                    name: "Del",
                    character: None,
                });
            }

            return Some(Key {
                pressed,
                name: "Keypad .",
                character: Some('.'),
            });
        }
        0x57 => {
            return Some(Key {
                pressed,
                name: "F11",
                character: None,
            });
        }
        0x58 => {
            return Some(Key {
                pressed,
                name: "F12",
                character: None,
            });
        }
        _ => key = None,
    }

    return key;
}
