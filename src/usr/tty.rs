use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};

use alloc::{
    alloc::{alloc, dealloc},
    format, str,
    string::String,
    vec::Vec,
};
use limine::{MemmapEntry, NonNullPtr};

use crate::{
    libs::{lazy::Lazy, mutex::Mutex},
    mem::LabelBytes,
};

pub struct Cursor {
    cx: AtomicU16,
    cy: AtomicU16,
    fg: AtomicU32,
    bg: AtomicU32,
}

pub struct Console {
    columns: AtomicU16,
    rows: AtomicU16,
    pub cursor: Cursor,
    feature_bits: Mutex<u8>,
    second_buffer: Mutex<Option<crate::drivers::video::Framebuffer>>,
}

struct ConsoleFeatures {
    _reserved: [u8; 6],
    serial_output: bool,
    graphical_output: bool,
    doubled_buffered: bool,
}

impl Console {
    #[inline]
    const fn new() -> Self {
        Self {
            columns: AtomicU16::new(0),
            rows: AtomicU16::new(0),
            cursor: Cursor::new(),
            feature_bits: Mutex::new(0b00000000),
            second_buffer: Mutex::new(None),
        }
    }

    #[inline]
    pub fn reinit(&self, back_buffer_region: Option<&NonNullPtr<MemmapEntry>>) {
        let framebuffer = crate::drivers::video::get_framebuffer();

        // Enable serial if it initialized correctly
        if crate::drivers::serial::POISONED.load(Ordering::SeqCst) == false {
            *self.feature_bits.lock().write() |= 1 << 1;
        }

        // Enable graphical output
        if framebuffer.is_some() {
            *self.feature_bits.lock().write() |= 1;
        } else {
            return;
        }

        if back_buffer_region.is_some() {
            *self.feature_bits.lock().write() |= 1 << 2;
            let mut back_buffer = crate::drivers::video::get_framebuffer().unwrap();

            back_buffer.pointer = back_buffer_region.unwrap().base as *mut u8;

            let row_size = back_buffer.pitch / (back_buffer.bpp / 8);

            let screen_size = row_size * back_buffer.height;

            unsafe {
                crate::arch::set_mtrr(
                    back_buffer_region.unwrap().base as u64,
                    screen_size as u64,
                    crate::arch::MTRRMode::WriteCombining,
                );

                core::ptr::write_bytes::<u32>(
                    back_buffer.pointer as *mut u32,
                    0x000000,
                    screen_size,
                );
            }

            (*self.second_buffer.lock().write()) = Some(back_buffer);
        }

        let framebuffer = framebuffer.unwrap();

        let columns = framebuffer.width / 8;
        let rows = framebuffer.height / 16;
        self.columns.swap(columns as u16, Ordering::SeqCst);
        self.rows.swap(rows as u16, Ordering::SeqCst);
    }

    fn get_features(&self) -> ConsoleFeatures {
        let graphical_output = ((*self.feature_bits.lock().read()) & 0x01) != 0;
        let serial_output = ((*self.feature_bits.lock().read()) & 0x02) != 0;
        let doubled_buffered = ((*self.feature_bits.lock().read()) & 0x04) != 0;

        return ConsoleFeatures {
            _reserved: [0; 6],
            serial_output,
            graphical_output,
            doubled_buffered,
        };
    }

    // Uses a stripped down version of ANSI color codes:
    // \033[FG;BGm
    pub fn puts(&self, string: &str) {
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
                            0 => {
                                self.cursor.set_fg(0xbababa);
                                self.cursor.set_bg(0x000000);
                            }
                            30..=37 => self.cursor.set_fg(color_to_hex(code - 30)),
                            40..=47 => self.cursor.set_bg(color_to_hex(code - 40)),
                            90..=97 => self.cursor.set_fg(color_to_hex(code - 30)),
                            100..=107 => self.cursor.set_bg(color_to_hex(code - 40)),
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

            if CONSOLE.get_features().serial_output {
                if character == '\n' {
                    crate::drivers::serial::write_serial('\r');
                }
                crate::drivers::serial::write_serial(character);
            }

            if !CONSOLE.get_features().graphical_output {
                // No graphical output, so to avoid errors, continue after sending serial
                continue;
            }

            if character == '\u{0008}' {
                CONSOLE.cursor.move_left();
                crate::drivers::serial::write_serial(' ');
                crate::drivers::serial::write_serial(character);
                crate::drivers::video::put_char(
                    ' ',
                    self.cursor.cx.load(Ordering::SeqCst),
                    self.cursor.cy.load(Ordering::SeqCst),
                    self.cursor.fg.load(Ordering::SeqCst),
                    self.cursor.bg.load(Ordering::SeqCst),
                    *self.second_buffer.lock().read(),
                );
                continue;
            }

            if character == '\r' {
                self.cursor
                    .set_pos(0, self.cursor.cy.load(Ordering::SeqCst));
                continue;
            }

            if character == '\n' {
                if (self.cursor.cy.load(Ordering::SeqCst) + 1) >= self.rows.load(Ordering::SeqCst) {
                    self.scroll_console();

                    self.cursor
                        .set_pos(0, self.cursor.cy.load(Ordering::SeqCst));
                } else {
                    self.cursor
                        .set_pos(0, self.cursor.cy.load(Ordering::SeqCst) + 1);
                }
            } else {
                crate::drivers::video::put_char(
                    character,
                    self.cursor.cx.load(Ordering::SeqCst),
                    self.cursor.cy.load(Ordering::SeqCst),
                    self.cursor.fg.load(Ordering::SeqCst),
                    self.cursor.bg.load(Ordering::SeqCst),
                    *self.second_buffer.lock().read(),
                );

                self.cursor.move_right();
            }
        }

        self.cursor.set_color(0xbababa, 0x000000);
    }

    pub fn scroll_console(&self) {
        let framebuffer_attributes = crate::drivers::video::get_framebuffer()
            .expect("Tried to scroll console but we have no framebuffer.");

        let lines_to_skip = 16;

        let framebuffer = framebuffer_attributes.pointer as *mut u32;

        let row_size = framebuffer_attributes.pitch / (framebuffer_attributes.bpp / 8);

        let screen_size = row_size * framebuffer_attributes.height;

        let skip = lines_to_skip * row_size;

        if self.get_features().doubled_buffered {
            let second_buffer = self.second_buffer.lock().read().unwrap().pointer as *mut u32;

            unsafe {
                core::ptr::copy_nonoverlapping(
                    second_buffer.add(skip),
                    second_buffer,
                    screen_size - skip,
                );

                crate::libs::util::memset32(
                    second_buffer.add(screen_size - skip) as *mut u32,
                    0x000000,
                    skip,
                );

                core::ptr::copy_nonoverlapping(second_buffer, framebuffer, screen_size);
            }
        } else {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    framebuffer.add(skip),
                    framebuffer,
                    screen_size - skip,
                );

                crate::libs::util::memset32(
                    framebuffer.add(screen_size - skip) as *mut u32,
                    0x000000,
                    skip,
                );
            }
        }
    }

    pub fn clear_screen(&self) {
        self.cursor.set_pos(0, 0);

        crate::drivers::video::fill_screen(
            self.cursor.bg.load(Ordering::SeqCst),
            *self.second_buffer.lock().read(),
        );
    }
}

// pub static CONSOLE: Console = Console::new();
pub static CONSOLE: Lazy<Console> = Lazy::new(|| {
    let console = Console::new();

    console.reinit(crate::mem::LARGEST_MEMORY_REGIONS.1);

    return console;
});

impl Cursor {
    #[inline]
    const fn new() -> Self {
        return Self {
            cx: AtomicU16::new(0),
            cy: AtomicU16::new(0),
            fg: AtomicU32::new(0xbababa),
            bg: AtomicU32::new(0x000000),
        };
    }

    pub fn set_pos(&self, new_cx: u16, new_cy: u16) {
        self.cx.swap(new_cx, Ordering::SeqCst);
        self.cy.swap(new_cy, Ordering::SeqCst);
    }

    fn move_right(&self) {
        let framebuffer_response = crate::drivers::video::FRAMEBUFFER_REQUEST
            .get_response()
            .get();

        if framebuffer_response.is_none() {
            return;
        }

        // eww, variable redeclaration
        let framebuffer_response = framebuffer_response.unwrap();
        let framebuffer = &framebuffer_response.framebuffers()[0];

        if self.cx.load(Ordering::SeqCst) == (framebuffer.width / 8) as u16 - 1 {
            if self.cy.load(Ordering::SeqCst) == (framebuffer.height / 16) as u16 - 1 {
                CONSOLE.scroll_console();

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

    fn move_left(&self) {
        let framebuffer_response = crate::drivers::video::FRAMEBUFFER_REQUEST
            .get_response()
            .get();

        if framebuffer_response.is_none() {
            return;
        }

        // eww, variable redeclaration
        let framebuffer_response = framebuffer_response.unwrap();
        let framebuffer = &framebuffer_response.framebuffers()[0];

        if self.cx.load(Ordering::SeqCst) == 0 {
            self.cx
                .swap((framebuffer.width / 8) as u16 - 1, Ordering::SeqCst);
            self.cy.fetch_sub(1, Ordering::SeqCst);
        } else {
            self.cx.fetch_sub(1, Ordering::SeqCst);
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

#[macro_export]
macro_rules! println {
    () => (crate::print!("\n"));
    ($($arg:tt)*) => (crate::print!("{}\n", &alloc::format!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        crate::usr::tty::CONSOLE.puts(&alloc::format!($($arg)*))
    )
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

static INPUT_BUFFER: Mutex<InputBuffer> = Mutex::new(InputBuffer { buffer: Vec::new() });

pub fn handle_key(key: crate::drivers::keyboard::Key) {
    let input_buffer = INPUT_BUFFER.lock().write();

    if !key.pressed {
        return;
    }

    if key.character == Some('\n') {
        CONSOLE.puts("\r\n");
        exec(input_buffer.as_str());
        input_buffer.clear();
        super::shell::prompt();
        return;
    }

    if key.name.starts_with("Cur") {
        if key.name.ends_with("Up") || key.name.ends_with("Down") {
            return;
        }

        if key.name.ends_with("Left") {
            CONSOLE.cursor.move_left();
            return;
        } else {
            CONSOLE.cursor.move_right();
            return;
        }
    }

    if key.character.is_none() {
        return;
    }

    if key.character.unwrap() == '\u{0003}' {
        CONSOLE.puts("^C\r\n");
        input_buffer.clear();
        super::shell::prompt();
        return;
    }

    if key.character.unwrap() == '\u{0008}' {
        if input_buffer.buffer.len() == 0 {
            return;
        }

        input_buffer.pop();
        CONSOLE.puts("\u{0008}");
        return;
    }

    let character = key.character.unwrap();
    input_buffer.push(character as u8);

    CONSOLE.puts(&format!("{}", key.character.unwrap()));
}

pub fn exec(command: &str) {
    let (command, args) = parse_input(command.trim());

    if command == "" {
        return;
    }

    if command == "memstat" {
        let allocator = &crate::mem::ALLOCATOR;

        let used_mem = allocator.inner.get_used_mem().label_bytes();
        let free_mem = allocator.inner.get_free_mem().label_bytes();
        let total_mem = allocator.inner.get_total_mem().label_bytes();

        println!(
            "Allocated so far: {used_mem}\nFree memory: {free_mem}\nTotal Memory: {total_mem}",
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
            println!("{mem:p} val: {}", unsafe { *(mem) });
        } else {
            // deallocate
            if args.len() < 3 {
                println!("Malformed input. See --help for detailed instructions.");
                return;
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

                println!("Value at memory address: {val}");
            }
        } else {
            println!("Argument provided is not a memory address.");
        }

        return;
    }

    if command == "memfill" {
        let allocator = &crate::mem::ALLOCATOR;
        let free_mem = allocator.inner.get_free_mem();

        unsafe {
            let layout = core::alloc::Layout::from_size_align(free_mem, 16).unwrap();
            let ptr = alloc(layout);
            dealloc(ptr, layout);
        }
        println!("Filled allocator with {free_mem} bytes");

        return;
    }

    if command == "echo" {
        let mut input = "";

        if args.len() != 0 {
            input = args[0].as_str();
        }

        CONSOLE.puts(input);
        CONSOLE.puts("\n");
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
        CONSOLE.clear_screen();
        return;
    }

    if command == "read" {
        if args.len() < 1 {
            println!("read: usage error: at least one argument is required!");
            return;
        }

        let file = crate::drivers::fs::vfs::VFS_INSTANCES.lock().read()[0].open(&args[0]);

        if file.is_err() {
            println!("read: Unable to read file!");
            return;
        }

        println!("{:X?}", file.unwrap().read());

        return;
    }

    if command == "test" {
        let message = "Hello from syscall!\n";
        unsafe {
            core::arch::asm!(
                "mov rdi, 0x01", // write syscall
                "mov rsi, 0x01", // stdio (but it doesnt matter)
                "mov rdx, {0:r}", // pointer
                "mov rcx, {1:r}", // count
                "int 0x80",
                in(reg) message.as_ptr(),
                in(reg) message.len()
            );
        }

        return;
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
