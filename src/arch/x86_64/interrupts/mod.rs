mod exceptions;

use crate::arch::x86_common::pic::ChainedPics;

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct IdtEntry {
    base_lo: u16,
    sel: u16,
    ist: u8,
    flags: u8,
    base_mid: u16,
    base_hi: u32,
    always0: u32,
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    base_lo: 0,
    sel: 0,
    ist: 0,
    always0: 0,
    flags: 0,
    base_hi: 0,
    base_mid: 0,
}; 256];

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static mut PICS: ChainedPics = ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET);

static mut IDT_PTR: IdtPtr = IdtPtr { limit: 0, base: 0 };

pub fn idt_set_gate(num: u8, function_ptr: u64, sel: u16, flags: u8) {
    let base = function_ptr;
    unsafe {
        IDT[num as usize] = IdtEntry {
            base_lo: (base & 0xFFFF) as u16,
            base_mid: ((base >> 16) & 0xFFFF) as u16,
            base_hi: ((base >> 32) & 0xFFFFFFFF) as u32,
            sel,
            ist: 0,
            always0: 0,
            flags,
        };
    }
}

extern "x86-interrupt" fn timer_handler() {
    // crate::usr::tty::puts(".");
    unsafe {
        PICS.notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

fn idt_init() {
    unsafe {
        let idt_size = core::mem::size_of::<IdtEntry>() * 256;
        IDT_PTR.limit = idt_size as u16 - 1;
        IDT_PTR.base = IDT.as_ptr() as u64;

        core::ptr::write_bytes(IDT.as_mut_ptr() as *mut core::ffi::c_void, 0, idt_size);

        // Set every interrupt to the default interrupt handler
        for num in 0..(idt_size) {
            idt_set_gate(num as u8, exceptions::generic_handler as u64, 0x28, 0xEE);
        }

        exceptions::set_exceptions();

        idt_set_gate(
            InterruptIndex::Timer.as_u8(),
            timer_handler as u64,
            0x28,
            0xEE,
        );

        core::arch::asm!(
            "lidt [{}]",
            "sti",
            in(reg) &IDT_PTR
        );

        crate::libs::logging::log_ok("Interrupt Descriptor Table");
    }
}

pub fn init() {
    idt_init();

    unsafe {
        PICS.initialize();
    }
}
