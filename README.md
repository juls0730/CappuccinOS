# CappuccinOS
CappuccinOS is a small x86-64 operating system written from scratch in rust. This README will guide you through the process of building and running CappuccinOS.

## Features
- [X] Serial output
- [X] Hardware interrupts
- [X] PS/2 Keyboard support
- [X] ANSI color codes in console
- [ ] Use APIC instead of PIC
- [ ] Roll my own bootloader
	- [ ] x86 CPU support
	- [ ] armv8 CPU support
- [ ] File system
  - [ ] IDE Device support
	- [ ] SATA device support
	- [ ] MMC/Nand device support
	- [ ] M.2 NVME device support
- [ ] Basic shell
  - [X] Basic I/O
	- [ ] Executing Programs
- [ ] Lua interpreter
- [ ] Multitasking
- [ ] Memory management
- [ ] Network support
- [ ] GUI
- [ ] Device drivers
	- [ ] Native intel graphics
- [ ] User authentication
- [ ] Power management
- [ ] Paging
- [ ] Heap allocation
- [ ] Hardware abstraction layer
- [ ] RTC Clock

## Setup
Before building CappuccinOS, make sure you have the following installed on your machine:

- rust
- binutils
- sgdisk
- mtools
- qemu (optional)

Clone the repo:
```BASH
git clone --recurse-submodules https://github.com/juls0730/CappuccinOS.git
cd CappuccinOS
```

Install rust, and switch to the nightly build:
```BASH
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain none
rustup override set nightly
```

## Usage
Build the image to `bin/CappuccinOS.iso`:
```BASH
make build
```

Run CappuccinOS with QEMU:
```BASH
make run
```

If you would like to target another architecture other than x86_64, set the `ARCH` variable to the a supported architecture. CappuccinOS is also built in release mode by default, if you would like to build CappuccinOS in debug mode, set the `MODE` variable to `debug`.

Run on a bare metal machine by flashing to a USB stick or hard drive:
```
sudo dd if=bin/CappuccinOS.iso of=/dev/sdX && sync
```
**Be careful not to overwrite your hard drive when using `dd`!**

## License
CappuccinOS is license under the MIT License. Feel free to modify and distribute in accordance with the license.