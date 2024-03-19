#![feature(abi_x86_interrupt, naked_functions)]
// Unforunately, this doesnt actually work with rust-analyzer, so if you want the annoying
// Error about "unnecessary returns" to go away, see https://github.com/rust-lang/rust-analyzer/issues/16542
// And if that issue ever gets closed, and you're reading this, feel free to remove this comment
#![allow(clippy::needless_return)]
#![no_std]
#![no_main]

use core::ffi::CStr;

use alloc::{format, vec::Vec};
use limine::KernelFileRequest;

use crate::drivers::fs::{
    initramfs,
    vfs::{vfs_open, UserCred},
};

extern crate alloc;

pub mod arch;
pub mod drivers;
pub mod libs;
pub mod mem;

pub static KERNEL_REQUEST: KernelFileRequest = KernelFileRequest::new(0);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    arch::interrupts::init();

    drivers::serial::init_serial();

    // let squashfs = initramfs::init();

    // crate::println!("{:?}", squashfs.superblock);

    let _ = drivers::fs::vfs::add_vfs("/", alloc::boxed::Box::new(initramfs::init()));

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    drivers::pci::enumerate_pci_bus();

    let mut file = vfs_open("/firstdir/seconddirbutlonger/yeah.txt").unwrap();

    // drivers::storage::ide::init();

    // let nested_file = vfs_open("/boot/limine/limine.cfg").unwrap();

    // crate::println!(
    //     "{:X?}",
    //     nested_file
    //         .ops
    //         .open(0, UserCred { uid: 0, gid: 0 }, nested_file.as_ptr())
    // );

    // let file = vfs_open("/example.txt").unwrap();
    crate::println!(
        "{:X?}",
        core::str::from_utf8(
            &file
                .ops
                .open(0, UserCred { uid: 0, gid: 0 }, file.as_ptr())
                .unwrap()
        )
        .unwrap()
    );

    let fb = drivers::video::get_framebuffer().unwrap();
    let length = (fb.height * fb.width) * (fb.bpp / 8);
    let pages = length / crate::mem::pmm::PAGE_SIZE;
    let buffer = unsafe {
        core::slice::from_raw_parts_mut(
            crate::mem::PHYSICAL_MEMORY_MANAGER
                .alloc(pages)
                .expect("Could not allocate color buffer") as *mut u32,
            length,
        )
    };

    for y in 0..fb.height {
        let r = ((y as f32) / ((fb.height - 1) as f32)) * 200.0;
        for x in 0..fb.width {
            let g = ((x as f32) / ((fb.width - 1) as f32)) * 200.0;
            buffer[y * fb.width + x] = ((r as u32) << 16) | ((g as u32) << 8) | 175;
        }
    }

    fb.blit_screen(buffer, None);

    // loop {
    //     let ch = read_serial();

    //     if ch == b'\x00' {
    //         continue;
    //     }

    //     if ch == b'\x08' {
    //         write_serial(b'\x08');
    //         write_serial(b' ');
    //         write_serial(b'\x08');
    //     }

    //     if ch > 0x20 && ch < 0x7F {
    //         write_serial(ch);
    //     }
    // }

    hcf();
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", &alloc::format!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (

        $crate::drivers::serial::write_string(&alloc::format!($($arg)*).replace('\n', "\n\r"))
    )
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => ($crate::println!("\x1B[97m[ \x1B[90m? \x1B[97m]\x1B[0m {}", &alloc::format!($($arg)*)));
}

#[macro_export]
macro_rules! log_serial {
    ($($arg:tt)*) => (
            $crate::drivers::serial::write_string(&alloc::format!($($arg)*).replace('\n', "\n\r"))
    );
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => ($crate::println!("\x1B[97m[ \x1B[91m! \x1B[97m]\x1B[0m {}", &alloc::format!($($arg)*)));
}

#[macro_export]
macro_rules! log_ok {
    ($($arg:tt)*) => ($crate::println!("\x1B[97m[ \x1B[92m* \x1B[97m]\x1B[0;m {}", &alloc::format!($($arg)*)));
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

pub static KERNEL_FEATURES: libs::cell::LazyCell<KernelFeatures> =
    libs::cell::LazyCell::new(parse_kernel_cmdline);

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
    let msg = &format!("{info}\n").replace('\n', "\n\r");

    drivers::serial::write_string(msg);

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

pub fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            core::arch::asm!("hlt");

            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            core::arch::asm!("wfi");
        }
    }
}
