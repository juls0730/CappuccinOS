#![feature(abi_x86_interrupt, allocator_api, naked_functions)]
#![no_std]
#![no_main]

extern crate alloc;

mod arch;
mod drivers;
mod libs;
mod mem;
mod usr;

use core::ffi::CStr;

use alloc::{format, vec::Vec};
use drivers::serial;
use libs::util::hcf;
use limine::KernelFileRequest;

use crate::{drivers::serial::write_serial, mem::LabelBytes};

pub static KERNEL_REQUEST: KernelFileRequest = KernelFileRequest::new(0);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    arch::interrupts::init();

    serial::init_serial();

    mem::log_info();

    drivers::acpi::init_acpi();

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    drivers::pci::enumerate_pci_bus();

    drivers::fs::vfs::init();

    // crate::println!("{:?}", INITRAMFS.open("/font.psf").unwrap().read());

    if let Some(kernel) = KERNEL_REQUEST.get_response().get() {
        crate::println!("{:X?}", kernel.kernel_file.get().unwrap().gpt_disk_uuid);
    }

    crate::println!(
        "Total memory: {}",
        crate::mem::PHYSICAL_MEMORY_MANAGER
            .total_memory()
            .label_bytes()
    );

    usr::shell::init_shell();

    hcf();
}

#[derive(Debug)]
pub struct KernelFeatures {
    pub fat_in_mem: bool,
}

impl KernelFeatures {
    fn update_option(&mut self, option: &str, value: &str) {
        #[allow(clippy::single_match)]
        match option {
            "fat_in_mem" => self.fat_in_mem = value == "true",
            _ => {}
        }
    }
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

    crate::println!("{kernel_arguments:?}");

    for item in kernel_arguments {
        let parts: Vec<&str> = item.split('=').collect();

        if parts.len() == 2 {
            let (option, value) = (parts[0], parts[1]);

            kernel_features.update_option(option, value);
        }
    }

    return kernel_features;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let message = format!("{}\n", info);

    for ch in message.chars() {
        write_serial(ch);
    }

    log_error!("{}", info);

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let rbp: u64;
        unsafe {
            core::arch::asm!("mov {0:r}, rbp", out(reg) rbp);
        };
        crate::arch::stack_trace::print_stack_trace(6, rbp);
    }

    hcf();
}
