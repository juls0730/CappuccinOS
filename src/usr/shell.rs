static mut INPUT_BUFFER: InputBuffer = InputBuffer {
    buffer: [0; BUFFER_SIZE],
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

const BUFFER_SIZE: usize = 256;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct InputBuffer {
    buffer: [u8; BUFFER_SIZE],
    length: usize,
}

impl InputBuffer {
    pub fn new() -> Self {
        InputBuffer {
            buffer: [0; BUFFER_SIZE],
            length: 0,
        }
    }

    pub fn clear(&mut self) {
        self.buffer = [0; BUFFER_SIZE];
        self.length = 0;
    }

    pub fn push(&mut self, value: u8) {
        if self.length < BUFFER_SIZE - 1 {
            self.buffer[self.length] = value;
            self.length += 1;
        }
    }

    pub fn pop(&mut self) {
        if self.length > 0 {
            self.length -= 1;
        }
    }

    pub fn as_str(&self) -> &str {
        // Convert the buffer to a string slice for convenience
        core::str::from_utf8(&self.buffer[0..self.length])
            .unwrap_or(core::str::from_utf8(b"").unwrap())
    }
}
