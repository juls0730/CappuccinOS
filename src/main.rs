#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![no_std]
#![no_main]

extern crate alloc;

mod arch;
mod drivers;
mod libs;
mod mem;
mod usr;

use drivers::serial;
use libs::{lazy::Lazy, mutex::Mutex, util::hcf};
use limine::ModuleRequest;

pub static MODULE_REQUEST: ModuleRequest = ModuleRequest::new(0);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    arch::interrupts::init();

    mem::log_info();

    serial::init_serial();

    // drivers::acpi::init_acpi();

    drivers::pci::enumerate_pci_bus();

    drivers::fs::vfs::init();

    if let Some(module_response) = MODULE_REQUEST.get_response().get() {
        let module_name = "initramfs.img";

        for module in module_response.modules() {
            let c_path = module.path.to_str();
            if c_path.is_none() {
                continue;
            }

            if !c_path.unwrap().to_str().unwrap().contains(module_name) {
                continue;
            }

            let initramfs = module;

            crate::println!("Initramfs is located at: {:#018X?}", unsafe {
                initramfs.base.as_ptr().unwrap()
                    ..initramfs
                        .base
                        .as_ptr()
                        .unwrap()
                        .add(initramfs.length as usize)
            });
        }
    }

    usr::shell::init_shell();

    hcf();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::log_error!("{}", info);

    hcf();
}
