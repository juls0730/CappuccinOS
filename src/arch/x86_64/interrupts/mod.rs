mod exceptions;

use crate::{arch::x86_common::pic::ChainedPics, libs::mutex::Mutex};

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    base_lo: u16,
    sel: u16,
    ist: u8,
    flags: u8,
    base_mid: u16,
    base_hi: u32,
    always0: u32,
}

impl IdtEntry {
    const fn new() -> Self {
        return Self {
            base_lo: 0,
            sel: 0,
            ist: 0,
            always0: 0,
            flags: 0,
            base_hi: 0,
            base_mid: 0,
        };
    }
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

static IDT: Mutex<[IdtEntry; 256]> = Mutex::new([IdtEntry::new(); 256]);

#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Ide = PIC_1_OFFSET + 14,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> = Mutex::new(ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET));

static mut IDT_PTR: IdtPtr = IdtPtr {
    limit: (core::mem::size_of::<IdtEntry>() * 256) as u16 - 1,
    base: 0,
};

pub fn idt_set_gate(num: u8, function_ptr: u64) {
    let base = function_ptr;
    IDT.lock().write()[num as usize] = IdtEntry {
        base_lo: (base & 0xFFFF) as u16,
        base_mid: ((base >> 16) & 0xFFFF) as u16,
        base_hi: ((base >> 32) & 0xFFFFFFFF) as u32,
        sel: 0x28,
        ist: 0,
        always0: 0,
        flags: 0xEE,
    };

    // If the interrupt with this number occurred with the "null" interrupt handler
    // We will need to tell the PIC that interrupt is over, this stops new interrupts
    // From never firing because "it was never finished"
    PICS.lock().write().notify_end_of_interrupt(num);
}

extern "x86-interrupt" fn null_interrupt_handler() {}

extern "x86-interrupt" fn timer_handler() {
    // crate::usr::tty::puts(".");
    PICS.lock()
        .write()
        .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
}

fn idt_init() {
    unsafe {
        let idt_size = core::mem::size_of::<IdtEntry>() * 256;
        IDT_PTR.base = IDT.lock().read().as_ptr() as u64;

        core::ptr::write_bytes(
            IDT.lock().write().as_mut_ptr() as *mut core::ffi::c_void,
            0,
            idt_size,
        );

        // Set every interrupt to the "null" interrupt handler (it does nothing)
        for num in 0..=255 {
            idt_set_gate(num, null_interrupt_handler as u64);
        }

        exceptions::set_exceptions();

        idt_set_gate(InterruptIndex::Timer.as_u8(), timer_handler as u64);
        idt_set_gate(0x80, syscall as u64);

        core::arch::asm!(
            "lidt [{}]",
            in(reg) &IDT_PTR
        );
    }
}

#[naked]
pub extern "C" fn syscall() {
    unsafe {
        core::arch::asm!(
            "push rdi",
            "push rsi",
            "push rdx",
            "push rcx",
            "call {}",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "iretq",
            options(noreturn),
            sym syscall_handler
        );
    }
}

pub extern "C" fn syscall_handler(rdi: u64, rsi: u64, rdx: u64, rcx: u64) {
    let buf = rdx as *const u8; // Treat as pointer to u8 (byte array)
    let count = rcx as usize;

    let slice = unsafe { core::slice::from_raw_parts(buf, count) };
    let message = core::str::from_utf8(slice).unwrap();
    crate::print!("{message}");
}

pub fn init() {
    idt_init();

    PICS.lock().write().initialize();
    unsafe {
        core::arch::asm!("sti");
    }
}
