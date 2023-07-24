use crate::{print, println};
use alloc::{borrow::ToOwned, format, vec::Vec};

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
                self.cx = (framebuffer.width / 8) as u16 - 2;
            } else {
                self.cx -= 1;
            }
        }
    }

    pub fn set_color(&mut self, new_fg: u32, new_bg: u32) {
        self.fg = new_fg;
        self.bg = new_bg;
    }
}

pub static mut CURSOR: Cursor = Cursor {
    cx: 0,
    cy: 0,
    fg: 0xbababa,
    bg: 0x000000,
};

// TODO: parse and use ANSI color codes
pub fn puts(string: &str) {
    if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
        .get_response()
        .get()
    {
        let framebuffer = &framebuffer_response.framebuffers()[0];

        for (_i, character) in string.chars().enumerate() {
            unsafe {
                if CURSOR.cx == (framebuffer.width / 8) as u16 - 1 {
                    CURSOR.set_pos(0, CURSOR.cy + 1);
                }
                // Newline character
                if character as u8 == 10 {
                    CURSOR.set_pos(0, CURSOR.cy + 1);
                } else {
                    crate::drivers::video::put_char(
                        character as u8,
                        CURSOR.cx,
                        CURSOR.cy,
                        CURSOR.fg,
                        CURSOR.bg,
                    );
                    CURSOR.set_pos(CURSOR.cx + 1, CURSOR.cy);
                }
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

pub fn handle_key(
    key: crate::drivers::keyboard::Key,
    input_buffer: &mut super::shell::InputBuffer,
    mods: crate::drivers::keyboard::ModStatuses,
) {
    if key.name == "Enter" || (mods.ctrl == true && key.name == "c") {
        puts("\n");
        exec(input_buffer.as_str());
        input_buffer.clear();
        super::shell::prompt();
        return;
    }

    if key.name == "Backspace" {
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
    let mut parts = command.trim().split_whitespace();
    let command = parts.next().unwrap_or("");
    let args = parts.collect::<Vec<&str>>();

    if command == "" {
        return;
    }

    if command == "memstat" {
        let allocator = &crate::libs::allocator::ALLOCATOR;
        println!(
            "Allocated so far: {}\nFree memory: {}",
            allocator.get_used(),
            allocator.get_free()
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
