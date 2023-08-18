use core::sync::atomic::{AtomicU8, Ordering};

use crate::drivers::keyboard::{set_leds, Key};

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
    status: AtomicU8,
    led_status: AtomicU8,
}

impl ModStatusBits {
    const fn new() -> Self {
        return Self {
            status: AtomicU8::new(0),
            led_status: AtomicU8::new(0),
        };
    }

    fn get_status(&self) -> ModStatus {
        let value = self.status.load(Ordering::SeqCst);

        return ModStatus {
            win: (value & 0b0000_0001) != 0,
            ctrl: (value & 0b0000_0010) != 0,
            alt: (value & 0b0000_0100) != 0,
            shift: (value & 0b0000_1000) != 0,
            caps: (value & 0b0001_0000) != 0,
            num_lock: (value & 0b0010_0000) != 0,
            scr_lock: (value & 0b0100_0000) != 0,
        };
    }

    fn set_modifier_key(&self, key: &str, status: bool) {
        let mut mod_status = self.get_status();
        let mut new_led_status = self.led_status.load(Ordering::SeqCst);

        match key {
            "win" => mod_status.win = status,
            "ctrl" => mod_status.ctrl = status,
            "alt" => mod_status.alt = status,
            "shift" => mod_status.shift = status,
            "caps" => {
                new_led_status ^= 0b00000100;
                mod_status.caps = status
            }
            "num_lock" => {
                new_led_status ^= 0b00000010;
                mod_status.num_lock = status
            }
            "scr_lock" => {
                new_led_status ^= 0b00000100;
                mod_status.scr_lock = status
            }
            _ => return,
        }

        // set Keyboard led (caps, num lock, scroll lock)
        set_leds(new_led_status);
        self.led_status.store(new_led_status, Ordering::SeqCst);

        let new_value = mod_status.to_byte();
        self.status.store(new_value, Ordering::SeqCst);
    }
}

static MOD_STATUS: ModStatusBits = ModStatusBits::new();

pub fn init_shell() {
    prompt();

    crate::drivers::keyboard::consume_scancode();
}

pub fn handle_key(mut key: Key) {
    if key.mod_key {
        parse_mod_key(&key);
    }

    if key.character.is_some() {
        key = parse_key(key);
    }

    super::tty::handle_key(key);
}

pub fn prompt() {
    super::tty::puts("> ");
}

fn parse_key(key: Key) -> Key {
    let mod_status = MOD_STATUS.get_status();
    let mut new_key = Key {
        mod_key: false,
        pressed: key.pressed,
        name: key.name,
        character: key.character,
    };

    if key.character.is_none() {
        panic!("Key passed into parse_key is not a character key!");
    }

    if mod_status.num_lock && key.name.starts_with("Keypad") {
        new_key = parse_keypad_keys(key);
        return new_key;
    }

    if mod_status.ctrl {
        new_key.character = Some(parse_unicode_keys(&key));
        return new_key;
    }

    if key.character.unwrap().is_alphabetic() && (mod_status.shift ^ mod_status.caps) {
        new_key.character = Some(key.character.unwrap().to_ascii_uppercase());
        return new_key;
    }

    if mod_status.shift && !key.name.starts_with("Keypad") {
        new_key.character = Some(capitalize_non_alphabetical(key.character.unwrap()));
        return new_key;
    }

    new_key.character = Some(key.character.unwrap());
    return new_key;
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

fn parse_keypad_keys(key: Key) -> Key {
    let mut new_key = Key {
        mod_key: false,
        pressed: key.pressed,
        name: key.name,
        character: key.character,
    };

    match key.character.unwrap() {
        '7' => {
            new_key.name = "Home";
            new_key.character = None;

            return new_key;
        }
        '8' => {
            new_key.name = "CurUp";
            new_key.character = None;

            return new_key;
        }
        '9' => {
            new_key.name = "PgUp";
            new_key.character = None;

            return new_key;
        }
        '4' => {
            new_key.name = "CurLeft";
            new_key.character = None;

            return new_key;
        }
        '5' => {
            new_key.character = None;

            return new_key;
        }
        '6' => {
            new_key.name = "CurRight";
            new_key.character = None;

            return new_key;
        }
        '1' => {
            new_key.name = "End";
            new_key.character = None;

            return new_key;
        }
        '2' => {
            new_key.name = "CurDown";
            new_key.character = None;

            return new_key;
        }
        '3' => {
            new_key.name = "PgDown";
            new_key.character = None;

            return new_key;
        }
        '0' => {
            new_key.name = "Insert";
            new_key.character = None;

            return new_key;
        }
        '.' => {
            new_key.name = "Del";
            new_key.character = None;

            return new_key;
        }
        _ => new_key,
    }
}

// bad name
fn parse_unicode_keys(key: &Key) -> char {
    assert!(key.character.is_some());

    match key.character.unwrap() {
        'c' => '\u{0003}',
        _ => key.character.unwrap(),
    }
}
