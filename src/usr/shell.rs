static mut INPUT_BUFFER: InputBuffer = InputBuffer {
    buffer: alloc::vec::Vec::new(),
    length: 0,
};

pub fn init_shell() {
    crate::drivers::keyboard::init_keyboard(handle_key);

    prompt();
}

fn handle_key(key: crate::drivers::keyboard::Key) {
    super::tty::handle_key(key, unsafe { &mut INPUT_BUFFER });
}

pub fn prompt() {
    super::tty::puts("> ");
}

pub struct InputBuffer {
    buffer: alloc::vec::Vec<u8>,
    length: usize,
}

impl InputBuffer {
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.length = 0;
    }

    pub fn push(&mut self, value: u8) {
        self.buffer.push(value);
        self.length += 1;
    }

    pub fn pop(&mut self) {
        if self.length > 0 {
						self.buffer.pop();
            self.length -= 1;
        }
    }

    pub fn as_str(&self) -> &str {
        // Convert the buffer to a string slice for convenience
        core::str::from_utf8(&self.buffer).unwrap_or("")
    }
}
