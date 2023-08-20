#![no_std]
#![no_main]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::format;

// piggyback off of the CappuccinOS allocator
// TODO: make a syscall for memory operations
#[allow(unused_imports)]
use CappuccinOS;

#[allow(dead_code)]
fn main(args: &[&str]) {
    let mut message = "Hello, World!\n".to_owned();

    if args.len() > 1 {
        message = format!("Hello, {}!\n", args[1]);
    }

    print(message.as_str());
}

fn print(message: &str) {
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
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    print("An exception occured!\n");
    loop {}
}
