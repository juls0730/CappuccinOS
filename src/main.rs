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

use core::ffi::CStr;

use alloc::vec::Vec;
use drivers::serial;
use libs::util::hcf;
use limine::KernelFileRequest;

use crate::mem::LabelBytes;

pub static KERNEL_REQUEST: KernelFileRequest = KernelFileRequest::new(0);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    arch::interrupts::init();

    serial::init_serial();

    mem::log_info();

    drivers::fs::initramfs::init();

    // drivers::acpi::init_acpi();

    drivers::pci::enumerate_pci_bus();

    drivers::fs::vfs::init();

    crate::println!(
        "Total memory: {}",
        crate::mem::PHYSICAL_MEMORY_MANAGER
            .total_memory()
            .label_bytes()
    );

    let asd: u32 = 0xdeadbeef;
    unsafe {
        *(asd as *mut u64) = 12;
    };

    usr::shell::init_shell();

    hcf();
}

#[derive(Debug)]
pub struct KernelFeatures {
    pub fat_in_mem: bool,
}

pub static KERNEL_FEATURES: libs::lazy::Lazy<KernelFeatures> =
    libs::lazy::Lazy::new(parse_kernel_cmdline);

fn parse_kernel_cmdline() -> KernelFeatures {
    let mut kernel_features: KernelFeatures = KernelFeatures { fat_in_mem: true };

    let kernel_file_response = KERNEL_REQUEST.get_response().get();
    if kernel_file_response.is_none() {
        return kernel_features;
    }

    let cmdline_ptr = kernel_file_response
        .unwrap()
        .kernel_file
        .get()
        .unwrap()
        .cmdline
        .as_ptr();

    if cmdline_ptr.is_none() {
        return kernel_features;
    }

    let cmdline = unsafe { CStr::from_ptr(cmdline_ptr.unwrap()) };
    let kernel_arguments = cmdline
        .to_str()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<&str>>();

    for &arg in kernel_arguments.iter() {
        let arg_parts = arg.split("=").collect::<Vec<&str>>();
        let mut arg_parts = arg_parts.iter().peekable();

        for _ in 0..arg_parts.len() {
            let part = arg_parts.next();
            if part.is_none() {
                break;
            }

            match part {
                Some(&"fat_in_mem") => {
                    if arg_parts.peek() == Some(&&"false") {
                        kernel_features.fat_in_mem = false;
                    }
                }
                _ => {}
            }
        }
    }

    return kernel_features;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::log_error!("{}", info);

    hcf();
}
