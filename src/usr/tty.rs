use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};

use alloc::{
    alloc::{alloc, dealloc},
    format, str,
    string::String,
    vec::Vec,
};
use limine::{Framebuffer, NonNullPtr};

pub struct Cursor {
    cx: AtomicU16,
    cy: AtomicU16,
    fg: AtomicU32,
    bg: AtomicU32,
}

impl Cursor {
    fn set_pos(&self, new_cx: u16, new_cy: u16) {
        self.cx.swap(new_cx, Ordering::SeqCst);
        self.cy.swap(new_cy, Ordering::SeqCst);
    }

    fn move_right(&self) {
        if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
            .get_response()
            .get()
        {
            let framebuffer = &framebuffer_response.framebuffers()[0];

            if self.cx.load(Ordering::SeqCst) == (framebuffer.width / 8) as u16 - 1 {
                if self.cy.load(Ordering::SeqCst) == (framebuffer.height / 16) as u16 - 1 {
                    scroll_console(framebuffer);

                    self.cy
                        .swap(((framebuffer.height / 16) - 1) as u16, Ordering::SeqCst);
                    self.cx.swap(0, Ordering::SeqCst);
                } else {
                    self.cy.fetch_add(1, Ordering::SeqCst);
                    self.cx.swap(0, Ordering::SeqCst);
                }
            } else {
                self.cx.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    fn move_left(&self) {
        if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
            .get_response()
            .get()
        {
            let framebuffer = &framebuffer_response.framebuffers()[0];

            if self.cx.load(Ordering::SeqCst) == 0 {
                self.cx
                    .swap((framebuffer.width / 8) as u16 - 1, Ordering::SeqCst);
                self.cy.fetch_sub(1, Ordering::SeqCst);
            } else {
                self.cx.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }

    pub fn set_fg(&self, new_fg: u32) {
        self.fg.swap(new_fg, Ordering::SeqCst);
    }

    pub fn set_bg(&self, new_bg: u32) {
        self.bg.swap(new_bg, Ordering::SeqCst);
    }

    pub fn set_color(&self, new_fg: u32, new_bg: u32) {
        self.fg.swap(new_fg, Ordering::SeqCst);
        self.bg.swap(new_bg, Ordering::SeqCst);
    }
}

pub static CURSOR: Cursor = Cursor {
    cx: AtomicU16::new(0),
    cy: AtomicU16::new(0),
    fg: AtomicU32::new(0xbababa),
    bg: AtomicU32::new(0x000000),
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
                        30..=37 => CURSOR.set_fg(color_to_hex(code - 30)),
                        40..=47 => CURSOR.set_bg(color_to_hex(code - 40)),
                        90..=97 => CURSOR.set_fg(color_to_hex(code - 30)),
                        100..=107 => CURSOR.set_bg(color_to_hex(code - 40)),
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

        if character == '\n' {
            if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
                .get_response()
                .get()
            {
                let framebuffer = &framebuffer_response.framebuffers()[0];

                // ! TODO: This copies excess data into the video buffer and thus,
                // leads to noise on my machine.
                if (CURSOR.cy.load(Ordering::SeqCst) + 1) >= (framebuffer.height / 16) as u16 {
                    scroll_console(framebuffer);

                    CURSOR.set_pos(0, CURSOR.cy.load(Ordering::SeqCst));
                } else {
                    CURSOR.set_pos(0, CURSOR.cy.load(Ordering::SeqCst) + 1);
                }
            }
        } else {
            crate::drivers::video::put_char(
                character,
                CURSOR.cx.load(Ordering::SeqCst),
                CURSOR.cy.load(Ordering::SeqCst),
                CURSOR.fg.load(Ordering::SeqCst),
                CURSOR.bg.load(Ordering::SeqCst),
            );

            CURSOR.move_right();
        }
    }

    CURSOR.set_color(0xbababa, 0x000000);
}

pub fn scroll_console(framebuffer: &NonNullPtr<Framebuffer>) {
    let size = framebuffer.pitch * framebuffer.height;

		// size / height = bytes per row of pixels, multiplied by 16 since font height is 16 pixels.
    let copy_from = ((size as f64 / framebuffer.height as f64) * 16.0) as usize;

		// Copy the framebuffer minus the top row to the framebuffer,
		// Then clear the last row with the background color to avoid noise on bare metal
		// See issue #1
    unsafe {
        core::ptr::copy(
            framebuffer.address.as_ptr().unwrap().add(copy_from),
            framebuffer.address.as_ptr().unwrap(),
            size as usize - copy_from,
        );

        crate::libs::util::memset32(framebuffer.address.as_ptr().unwrap().add(size as usize - copy_from) as *mut u32, CURSOR.bg.load(Ordering::SeqCst), copy_from);
    }
}

pub fn clear_screen() {
    CURSOR.set_pos(0, 0);

    crate::drivers::video::fill_screen(CURSOR.bg.load(Ordering::SeqCst));
}

#[macro_export]
macro_rules! println {
    () => (crate::print!("\n"));
    ($($arg:tt)*) => (crate::print!("{}\n", &format!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (crate::usr::tty::puts(&format!($($arg)*)));
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

pub fn handle_key(key: crate::drivers::keyboard::Key) {
    let input_buffer = unsafe { &mut INPUT_BUFFER };

    if key.name == "Enter" {
        puts("\n");
        exec(input_buffer.as_str());
        input_buffer.clear();
        super::shell::prompt();
        return;
    }

    if key.name == "Backspace" && input_buffer.buffer.len() > 0 {
        input_buffer.pop();
        CURSOR.move_left();
        puts(" ");
        CURSOR.move_left();
        return;
    }

    if key.name.starts_with("Cur") {
        if key.name.ends_with("Up") || key.name.ends_with("Down") {
            return;
        }

        if key.name.ends_with("Left") {
            CURSOR.move_left();
            return;
        } else {
            CURSOR.move_right();
            return;
        }
    }

    if key.character.is_some() {
        if key.character.unwrap() == '\u{0003}' {
            puts("^C\n");
            input_buffer.clear();
            super::shell::prompt();
            return;
        }

        let character = key.character.unwrap();
        input_buffer.push(character as u8);

        puts(&format!("{}", key.character.unwrap()));
    }
}

pub fn exec(command: &str) {
    let (command, args) = parse_input(command.trim());

    if command == "" {
        return;
    }

    if command == "memstat" {
        let allocator = &crate::sys::mem::ALLOCATOR;

        let (used_mem, used_mem_label) = label_units(allocator.get_used_mem());
        let (free_mem, free_mem_label) = label_units(allocator.get_free_mem());
        let (total_mem, total_mem_label) = label_units(allocator.get_total_mem());

        println!(
            "Allocated so far: {used_mem} {used_mem_label}\nFree memory: {free_mem} {free_mem_label}\nTotal Memory: {total_mem} {total_mem_label}",
        );
        return;
    }

    if command == "memalloc" {
        if args.len() == 0 {
            println!("Allocation size is required. See --help for detailed instructions.");
            return;
        }

        if args[0].as_str() == "--help" || args[0].as_str() == "-h" {
            // print help menu
            println!("memalloc ALLOCATION_SIZE [OPTIONS]\n-d alias: --dealloc; Deallocates memory at the specified location with specified size.");
            return;
        }

        if args.len() == 1 {
            // allocate
            let size: Result<usize, core::num::ParseIntError> = args[0].as_str().parse();

            if size.is_err() {
                println!(
                    "Provided argument is not a number. See --help for detailed instructions."
                );
                return;
            }

            let layout = core::alloc::Layout::from_size_align(size.unwrap(), 16).unwrap();

            let mem = unsafe { alloc(layout) as *mut u16 };
            unsafe { *(mem as *mut u16) = 42 };
            println!("{:p} val: {}", mem, unsafe { *(mem) });
        } else {
            // deallocate
            if args.len() < 3 {
                println!("Malformed input. See --help for detailed instructions.");
            }

            let mut memory_address = 0;
            let mut size = 0;

            for arg in args {
                if arg.starts_with("-") {
                    continue;
                }

                if arg.starts_with("0x") {
                    memory_address = parse_memory_address(arg.as_str()).unwrap();
                    continue;
                }

                let num_arg = arg.parse::<usize>();

                if num_arg.is_err() {
                    println!(
                        "Provided argument is not a number. See --help for detailed instructions."
                    );
                    return;
                }

                size = num_arg.unwrap();
            }

            let layout = core::alloc::Layout::from_size_align(size, 16).unwrap();

            let ptr = memory_address as *mut u8;

            unsafe {
                dealloc(ptr, layout);

                println!("Deallocated memory at address: {:?}", ptr);
            }
        }
        return;
    }

    if command == "memtest" {
        if args.len() == 0 {
            println!("Memory address to test is required.");
            return;
        }

        let arg = args[0].as_str();

        if let Some(addr) = parse_memory_address(arg) {
            let ptr: *const u32 = addr as *const u32;

            unsafe {
                let val = *ptr;

                println!("Value at memory address: {}", val);
            }
        } else {
            println!("Argument provided is not a memory address.");
        }

        return;
    }

    if command == "memmap" {
        crate::sys::mem::memory_map_info();
        return;
    }

    if command == "memfill" {
        let allocator = &crate::sys::mem::ALLOCATOR;
        let free_mem = allocator.get_free_mem();

        unsafe {
            let layout = core::alloc::Layout::from_size_align(free_mem, 16).unwrap();
            let ptr = alloc(layout);
            dealloc(ptr, layout);
        }
        println!("Filled allocator with {} bytes", free_mem);

        return;
    }

    if command == "echo" {
        let mut input = "";

        if args.len() != 0 {
            input = args[0].as_str();
        }

        puts(input);
        puts("\n");
        return;
    }

    if command == "poke" {
        if args.len() < 2 {
            println!("poke: usage error: memory address & value required!");
            return;
        }

        if let Some(addr) = parse_memory_address(args[0].as_str()) {
            let value: Result<u32, core::num::ParseIntError> = args[1].as_str().parse();

            if value.is_err() {
                println!("Second argument provided is not a number.");
            }

            let ptr: *mut u32 = addr as *mut u32;

            unsafe {
                *ptr = value.unwrap();

                println!("Allocated {:?} at {:#x}", *ptr, addr);
            }
        } else {
            println!("First argument provided is not a memory address.");
        }
    }

    if command == "clear" {
        clear_screen();
        return;
    }

    if command == "test" {
        if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
            .get_response()
            .get()
        {
            let framebuffer = &framebuffer_response.framebuffers()[0];

            println!("{:?}", framebuffer);
            return;
        }
    }

    println!("{:?} {:?}", command, args);
}

fn parse_input(input: &str) -> (String, Vec<String>) {
    let mut command = String::new();
    let mut args: Vec<String> = Vec::new();
    let mut iter = input.trim().chars().peekable();

    let mut i: usize = 0;
    while let Some(char) = iter.next() {
        let mut arg = String::new();

        match char {
            ' ' => continue,
            '"' | '\'' => {
                let mut escape_char = '"';
                if char == '\'' {
                    escape_char = '\'';
                }

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
                if char == '\\' {
                    if let Some(ch) = iter.next() {
                        arg.push(parse_escaped_char(ch));
                    }
                } else {
                    arg.push(char);
                }

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
        '0' => '\0',
        _ => next_char, // You can add more escape sequences if needed
    };
    return escaped;
}

fn parse_memory_address(input: &str) -> Option<u64> {
    if input.starts_with("0x") {
        u64::from_str_radix(&input[2..], 16).ok()
    } else {
        None
    }
}

fn label_units(bytes: usize) -> (usize, &'static str) {
    if bytes >> 30 > 0 {
        return (bytes >> 30, "GiB");
    } else if bytes >> 20 > 0 {
        return (bytes >> 20, "MiB");
    } else if bytes >> 10 > 0 {
        return (bytes >> 10, "KiB");
    } else {
        return (bytes, "Bytes");
    }
}
