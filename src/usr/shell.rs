#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::drivers::keyboard::set_leds;
use crate::{
    drivers::keyboard::Key,
    libs::{bit_manipulator::BitManipulator, mutex::Mutex},
};

struct ModStatus {
    pub win: bool,      // first bit
    pub ctrl: bool,     // second bit
    pub alt: bool,      // third bit
    pub shift: bool,    // forth bit
    pub caps: bool,     // fifth bit
    pub num_lock: bool, // sixth bit
    pub scr_lock: bool, // (possibly unnecessary) seventh bit
}

impl ModStatus {
    fn to_byte(&self) -> u8 {
        let mut value = 0u8;
        if self.win {
            value |= 0b0000_0001;
        }
        if self.ctrl {
            value |= 0b0000_0010;
        }
        if self.alt {
            value |= 0b0000_0100;
        }
        if self.shift {
            value |= 0b0000_1000;
        }
        if self.caps {
            value |= 0b0001_0000;
        }
        if self.num_lock {
            value |= 0b0010_0000;
        }
        if self.scr_lock {
            value |= 0b0100_0000;
        }
        return value;
    }
}

struct ModStatusBits {
    status: Mutex<BitManipulator<u8>>,
    led_status: Mutex<BitManipulator<u8>>,
}

impl ModStatusBits {
    const fn new() -> Self {
        return Self {
            status: Mutex::new(BitManipulator::<u8>::new()),
            led_status: Mutex::new(BitManipulator::<u8>::new()),
        };
    }

    fn get_status(&self) -> ModStatus {
        let status = self.status.lock().read();

        return ModStatus {
            win: status.extract_bit(0),
            ctrl: status.extract_bit(1),
            alt: status.extract_bit(2),
            shift: status.extract_bit(3),
            caps: status.extract_bit(4),
            num_lock: status.extract_bit(5),
            scr_lock: status.extract_bit(6),
        };
    }

    fn set_modifier_key(&self, key: &str, status: bool) {
        let mut led_status = *self.led_status.lock().write();
        let mut mod_status = self.get_status();

        match key {
            "win" => mod_status.win = status,
            "ctrl" => mod_status.ctrl = status,
            "alt" => mod_status.alt = status,
            "shift" => mod_status.shift = status,
            "caps" => {
                led_status ^= 0b00000100;
                mod_status.caps = status
            }
            "num_lock" => {
                led_status ^= 0b00000010;
                mod_status.num_lock = status
            }
            "scr_lock" => {
                led_status ^= 0b00000100;
                mod_status.scr_lock = status
            }
            _ => return,
        }

        // set Keyboard led (caps, num lock, scroll lock)
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        set_leds(led_status.get());

        let new_value = mod_status.to_byte();
        self.status.lock().write().set(new_value);
    }
}

static MOD_STATUS: ModStatusBits = ModStatusBits::new();

pub fn init_shell() {
    prompt();

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    crate::drivers::keyboard::consume_scancode();
}

pub fn handle_key(mut key: Key) {
    if key.name.len() > 1 && key.character.is_none() {
        parse_mod_key(&key);
    }

    if key.character.is_some() {
        key = parse_key(key);
    }

    super::tty::handle_key(key);
}

pub fn prompt() {
    super::tty::CONSOLE.puts("> ");
}

fn parse_key(mut key: Key) -> Key {
    let mod_status = MOD_STATUS.get_status();

    if key.character.is_none() {
        panic!("Key passed into parse_key is not a character key!");
    }

    if mod_status.num_lock && key.name.starts_with("Keypad") {
        key = parse_keypad_keys(key);
        return key;
    }

    if mod_status.ctrl {
        key.character = Some(parse_unicode_keys(&key));
        return key;
    }

    if key.character.unwrap().is_alphabetic() && (mod_status.shift ^ mod_status.caps) {
        key.character = Some(key.character.unwrap().to_ascii_uppercase());
        return key;
    }

    if mod_status.shift && !key.name.starts_with("Keypad") {
        key.character = Some(capitalize_non_alphabetical(key.character.unwrap()));
        return key;
    }

    key.character = Some(key.character.unwrap());
    return key;
}

fn capitalize_non_alphabetical(character: char) -> char {
    match character {
        '`' => '~',
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '0' => ')',
        '-' => '_',
        '=' => '+',
        '[' => '{',
        ']' => '}',
        '\\' => '|',
        ';' => ':',
        '\'' => '"',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        _ => character,
    }
}

fn parse_mod_key(key: &Key) {
    // Held mod keys
    if key.name.ends_with("Ctrl") {
        MOD_STATUS.set_modifier_key("ctrl", key.pressed);
        return;
    }

    if key.name.ends_with("Shift") {
        MOD_STATUS.set_modifier_key("shift", key.pressed);
        return;
    }

    if key.name.ends_with("Alt") {
        MOD_STATUS.set_modifier_key("alt", key.pressed);
        return;
    }

    // Toggled mod keys
    if !key.pressed {
        return;
    }

    let mod_status = MOD_STATUS.get_status();

    if key.name == "CapsLock" {
        MOD_STATUS.set_modifier_key("caps", !mod_status.caps);
        return;
    }

    if key.name == "NumLock" {
        MOD_STATUS.set_modifier_key("num_lock", !mod_status.num_lock);
        return;
    }
}

fn parse_keypad_keys(mut key: Key) -> Key {
    match key.character.unwrap() {
        '7' => {
            key.name = "Home";
        }
        '8' => {
            key.name = "CurUp";
        }
        '9' => {
            key.name = "PgUp";
        }
        '4' => {
            key.name = "CurLeft";
        }
        // 5 has no special function
        '6' => {
            key.name = "CurRight";
        }
        '1' => {
            key.name = "End";
        }
        '2' => {
            key.name = "CurDown";
        }
        '3' => {
            key.name = "PgDown";
        }
        '0' => {
            key.name = "Insert";
        }
        '.' => {
            key.name = "Del";
        }
        _ => {}
    };

    key.character = None;
    return key;
}

// bad name
fn parse_unicode_keys(key: &Key) -> char {
    assert!(key.character.is_some());

    match key.character.unwrap() {
        'c' => '\u{0003}',
        _ => key.character.unwrap(),
    }
}
