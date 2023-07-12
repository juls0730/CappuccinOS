struct Cursor {
	cx: u16,
	cy: u16,
	fg: u32,
	bg: u32,
}

static mut CURSOR: Cursor = Cursor{ 
	cx: 0, 
	cy: 0,
	fg: 0xbababa,
	bg: 0x000000,
};

pub fn puts(string: &str) {
	if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST.get_response().get() {
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
					crate::drivers::video::put_char(character as u8, CURSOR.cx, CURSOR.cy, CURSOR.fg, CURSOR.bg);
					CURSOR.cx += 1;
				}
			}
		}
	}
}

fn move_cursor_left() {
	if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST.get_response().get() {
		let framebuffer = &framebuffer_response.framebuffers()[0];

		unsafe {
			if CURSOR.cx == 0 {
				CURSOR.cy -= 1;
				CURSOR.cx = (framebuffer.width / 8) as u16 - 1;
			} else {
				CURSOR.cx -= 1;
			}
		}
	}
}

fn move_cursor_right() {
	if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST.get_response().get() {
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

pub fn handle_key(key: crate::drivers::keyboard::Key) {
	if key.key == "Enter" {
		puts("\n");
		super::shell::prompt();
	}

	if key.key == "Backspace" {
		move_cursor_left();
		puts(" ");
		move_cursor_left();
	}

	if key.key.starts_with("Cur") {
		if key.key.ends_with("Up") || key.key.ends_with("Down") {
			return;
		}

		if key.key.ends_with("Left") {
			move_cursor_left();
		} else {
			move_cursor_right();
		}
	}

	if key.printable {
		puts(key.key);
	}
}