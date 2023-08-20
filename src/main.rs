#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![no_std]
#![no_main]

extern crate alloc;

mod arch;
mod drivers;
mod libs;
mod sys;
mod usr;

use alloc::format;
use drivers::serial;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    sys::mem::init();

    serial::init_serial();

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    arch::interrupts::init();

    usr::shell::init_shell();

    hcf();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    libs::logging::log_error(&format!("{}", info));

    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            core::arch::asm!("hlt");

            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfi");
        }
    }
}
