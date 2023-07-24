use alloc::{borrow::ToOwned, str, vec::Vec};

static mut INPUT_BUFFER: InputBuffer = InputBuffer { buffer: Vec::new() };

pub fn init_shell() {
    crate::drivers::keyboard::init_keyboard(handle_key);

    prompt();
}

fn handle_key(key: crate::drivers::keyboard::Key, mods: crate::drivers::keyboard::ModStatuses) {
    super::tty::handle_key(key, unsafe { &mut INPUT_BUFFER }, mods);
}

pub fn prompt() {
    super::tty::puts("> ");
}

pub struct InputBuffer {
    buffer: Vec<u8>,
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
