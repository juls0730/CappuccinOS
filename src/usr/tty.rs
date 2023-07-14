use alloc::format;

struct Cursor {
    cx: u16,
    cy: u16,
    fg: u32,
    bg: u32,
}

static mut CURSOR: Cursor = Cursor {
    cx: 0,
    cy: 0,
    fg: 0xbababa,
    bg: 0x000000,
};

pub fn puts(string: &str) {
    if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
        .get_response()
        .get()
    {
        let framebuffer = &framebuffer_response.framebuffers()[0];

        for (_i, character) in string.chars().enumerate() {
            unsafe {
                if CURSOR.cx == (framebuffer.width / 8) as u16 - 1 {
                    CURSOR.cy += 1;
                    CURSOR.cx = 0;
                }
                // Newline character
                if character as u8 == 10 {
                    CURSOR.cx = 0;
                    CURSOR.cy += 1;
                } else {
                    crate::drivers::video::put_char(
                        character as u8,
                        CURSOR.cx,
                        CURSOR.cy,
                        CURSOR.fg,
                        CURSOR.bg,
                    );
                    CURSOR.cx += 1;
                }
            }
        }
    }
}

fn move_cursor_left() {
    if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
        .get_response()
        .get()
    {
        let framebuffer = &framebuffer_response.framebuffers()[0];

        unsafe {
            if CURSOR.cx == 0 {
                CURSOR.cy -= 1;
								puts(&alloc::format!("{}", CURSOR.cy));
                CURSOR.cx = (framebuffer.width / 8) as u16 - 2;
            } else {
                CURSOR.cx -= 1;
            }
        }
    }
}

fn move_cursor_right() {
    if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
        .get_response()
        .get()
    {
        let framebuffer = &framebuffer_response.framebuffers()[0];

        unsafe {
            if CURSOR.cx == (framebuffer.width / 8) as u16 - 1 {
                CURSOR.cy += 1;
                CURSOR.cx = 0;
            } else {
                CURSOR.cx += 1;
            }
        }
    }
}

pub fn set_color(color: u32) {
    unsafe {
        CURSOR.fg = color;
    }
}

pub fn handle_key(
    key: crate::drivers::keyboard::Key,
    input_buffer: &mut super::shell::InputBuffer,
) {
    if key.name == "Enter" {
        puts("\n");
        exec(input_buffer.as_str());
        input_buffer.clear();
        super::shell::prompt();
    }

    if key.name == "Backspace" {
        input_buffer.pop();
        move_cursor_left();
        puts(" ");
        move_cursor_left();
    }

    if key.name.starts_with("Cur") {
        if key.name.ends_with("Up") || key.name.ends_with("Down") {
            return;
        }

        if key.name.ends_with("Left") {
            move_cursor_left();
        } else {
            move_cursor_right();
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
    let args = parts;

    if command == "" {
        return;
    }

    puts(&format!("{}\n", command));
}
