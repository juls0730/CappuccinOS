#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use CappuccinOS::api::syscall;
use CappuccinOS::entry_point;

entry_point!(main);

fn main(args: &[&str]) {
    if args.len() > 1 {
        // FIXME: This will result in a page fault exception for an address
        // that's already mapped to the kernel stack
        syscall::write(&format!("Hello, {}!\n", args[1]));
    } else {
        syscall::write("Hello, World!\n");
    }
}