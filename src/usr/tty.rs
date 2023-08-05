use crate::{print, println};
use alloc::{borrow::ToOwned, format, str, string::String, vec::Vec};

pub struct Cursor {
    cx: u16,
    cy: u16,
    fg: u32,
    bg: u32,
}

impl Cursor {
    fn set_pos(&mut self, new_cx: u16, new_cy: u16) {
        self.cx = new_cx;
        self.cy = new_cy;
    }

    fn move_right(&mut self) {
        if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
            .get_response()
            .get()
        {
            let framebuffer = &framebuffer_response.framebuffers()[0];

            if self.cx == (framebuffer.width / 8) as u16 - 1 {
                self.cy += 1;
                self.cx = 0;
            } else {
                self.cx += 1;
            }
        }
    }

    fn move_left(&mut self) {
        if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
            .get_response()
            .get()
        {
            let framebuffer = &framebuffer_response.framebuffers()[0];

            if self.cx == 0 {
                self.cx = (framebuffer.width / 8) as u16 - 1;
                self.cy -= 1;
            } else {
                self.cx -= 1;
            }
        }
    }

    pub fn set_fg(&mut self, new_fg: u32) {
        self.fg = new_fg;
    }

    pub fn set_bg(&mut self, new_bg: u32) {
        self.bg = new_bg;
    }
}

pub static mut CURSOR: Cursor = Cursor {
    cx: 0,
    cy: 0,
    fg: 0xbababa,
    bg: 0x000000,
};

fn color_to_hex(color: u8) -> u32 {
    match color {
        0 => 0x000000,
        1 => 0xCD0000,
        2 => 0x00CD00,
        3 => 0xCDCD00,
        4 => 0x0000EE,
        5 => 0xCD00CD,
        6 => 0x00CDCD,
        7 => 0xBABABA,
        60 => 0x555555,
        61 => 0xFF0000,
        62 => 0x00FF00,
        63 => 0xFFFF00,
        64 => 0x5C5CFF,
        65 => 0xFF00FF,
        66 => 0x00FFFF,
        67 => 0xFFFFFF,
        _ => 0x000000,
    }
}

// Uses a stripped down version of ANSI color codes:
// \033[FG;BGm
pub fn puts(string: &str) {
    let mut in_escape_sequence = false;
    let mut color_code_buffer = String::new();

    for (_i, character) in string.chars().enumerate() {
        if in_escape_sequence {
            if character == 'm' {
                in_escape_sequence = false;

                let codes: Vec<u8> = color_code_buffer
                    .split(';')
                    .filter_map(|code| code.parse().ok())
                    .collect();

                for code in codes {
                    match code {
                        30..=37 => unsafe { CURSOR.set_fg(color_to_hex(code - 30)) },
                        40..=47 => unsafe { CURSOR.set_bg(color_to_hex(code - 40)) },
                        90..=97 => unsafe { CURSOR.set_fg(color_to_hex(code - 30)) },
                        100..=107 => unsafe { CURSOR.set_bg(color_to_hex(code - 40)) },
                        _ => {}
                    }
                }

                color_code_buffer.clear();
            } else if character.is_ascii_digit() || character == ';' {
                color_code_buffer.push(character);
            } else {
                if character == '[' {
                    // official start of the escape sequence
                    color_code_buffer.clear();
                    continue;
                }

                in_escape_sequence = false;
                color_code_buffer.clear();
            }

            continue;
        }

        if character == '\0' {
            in_escape_sequence = true;
            continue;
        }

        unsafe {
            if character == '\n' {
                CURSOR.set_pos(0, CURSOR.cy + 1);
            } else {
                crate::drivers::video::put_char(
                    character, CURSOR.cx, CURSOR.cy, CURSOR.fg, CURSOR.bg,
                );
                CURSOR.move_right();
            }
        }
    }
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", &format!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (puts(&format!($($arg)*)));
}

pub struct InputBuffer {
    pub buffer: Vec<u8>,
}

impl InputBuffer {
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn push(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn pop(&mut self) {
        if self.buffer.len() > 0 {
            self.buffer.pop();
        }
    }

    pub fn as_str(&self) -> &str {
        // Convert the buffer to a string slice for convenience
        str::from_utf8(&self.buffer).unwrap_or("")
    }
}

static mut INPUT_BUFFER: InputBuffer = InputBuffer { buffer: Vec::new() };

pub fn handle_key(key: crate::drivers::keyboard::Key, mods: crate::drivers::keyboard::ModStatuses) {
    let input_buffer = unsafe { &mut INPUT_BUFFER };

    if key.name == "Enter" || (mods.ctrl == true && key.name == "c") {
        puts("\n");
        exec(input_buffer.as_str());
        input_buffer.clear();
        super::shell::prompt();
        return;
    }

    if key.name == "Backspace" && input_buffer.buffer.len() > 0 {
        input_buffer.pop();
        unsafe {
            CURSOR.move_left();
        }
        puts(" ");
        unsafe {
            CURSOR.move_left();
        }
        return;
    }

    if key.name.starts_with("Cur") {
        if key.name.ends_with("Up") || key.name.ends_with("Down") {
            return;
        }

        if key.name.ends_with("Left") {
            unsafe {
                CURSOR.move_left();
            }
            return;
        } else {
            unsafe {
                CURSOR.move_left();
            }
            return;
        }
    }

    if key.printable {
        let character = key.name.chars().next().unwrap_or('\0');
        input_buffer.push(character as u8);

        puts(key.name);
    }
}

pub fn exec(command: &str) {
    let (command, args) = parse_input(command.trim());

    if command == "" {
        return;
    }

    if command == "memstat" {
        let allocator = &crate::libs::allocator::ALLOCATOR;
        println!(
            "Allocated so far: {}\nFree memory: {}",
            allocator.get_used_mem(),
            allocator.get_free_mem()
        );
        return;
    }

    print!("{} ", command);
    print!("[");
    for (i, arg) in args.iter().enumerate() {
        print!("{}", arg);
        if i != args.len() - 1 {
            print!(", ");
        }
    }
    println!("]");
}

fn parse_input(input: &str) -> (String, Vec<String>) {
    let mut command = String::new();
    let mut args: Vec<String> = Vec::new();
    let mut iter = input.trim().chars().peekable();

    let mut i: usize = 0;
    while let Some(char) = iter.next() {
        match char {
            ' ' => continue,
            '"' | '\'' => {
                let mut escape_char = '"';
                if char == '\'' {
                    escape_char = '\'';
                }
                let mut arg = String::new();

                while let Some(ch) = iter.next() {
                    match ch {
                        '\\' => {
                            if let Some(next_char) = iter.next() {
                                arg.push(parse_escaped_char(next_char));
                            }
                        }
                        '"' | '\'' => {
                            if ch == escape_char {
                                break;
                            }

                            arg.push(ch);
                        }
                        _ => arg.push(ch),
                    }
                }

                if i == 0 {
                    command = arg;
                } else {
                    args.push(arg);
                }
            }
            _ => {
                let mut arg = String::new();
                arg.push(char);

                while let Some(ch) = iter.peek() {
                    match ch {
                        &' ' | &'"' | &'\'' => break,
												&'\\' => {
													iter.next();
													if let Some(next_char) = iter.next() {
														arg.push(parse_escaped_char(next_char));
													}
												}
                        _ => arg.push(iter.next().unwrap()),
                    }
                }

                if i == 0 {
                    command = arg;
                } else {
                    args.push(arg);
                }
            }
        }
        i += 1;
    }

    return (command, args);
}

fn parse_escaped_char(next_char: char) -> char {
    let escaped = match next_char {
        'n' => '\n',
        't' => '\t',
        '\\' => '\\',
        '\'' => '\'',
        '"' => '"',
        _ => next_char, // You can add more escape sequences if needed
    };
    return escaped;
}
