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

use drivers::serial;
use libs::util::hcf;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    arch::interrupts::init();

    sys::mem::init();

    serial::init_serial();

    // drivers::acpi::init_acpi();

    drivers::pci::enumerate_pci_bus();

    drivers::ide::init();

    usr::shell::init_shell();

    hcf();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log_error!("{}", info);

    hcf();
}
