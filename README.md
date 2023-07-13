# Toto OS
Toto OS is a small x86-64 operating system written from scratch in rust. This README will guide you through the process of building and running Toto OS.

## Features
- [X] Serial output
- [X] Hardware interrupts
- [X] PS/2 Keyboard support
- [ ] Use APIC instead of PIC
- [ ] Roll my own bootloader
	- [ ] x86 CPU support
	- [ ] armv8 CPU support
- [ ] ANSI color codes in console
- [ ] File system
- [ ] Basic shell
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
Before building Toto OS, make sure you have the following installed on your machine:

- rust
- binutils
- parted
- qemu (optional)

Clone the repo:
```BASH
git clone --recurse-submodules https://github.com/juls0730/toto-os.git
cd toto-os
```

Install rust, and switch to the nightly build:
```BASH
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain none
rustup override set nightly
```

## Usage
Build the image to `bin/toto-os.iso`:
```BASH
make build
```

Run Toto OS with QEMU:
```BASH
make run
```

Run on a bare metal machine by flashing to a USB stick or hard drive:
```
sudo dd if=bin/toto-os.iso of=/dev/sdX && sync
```
**Be careful not to overwrite your hard drive when using `dd`!**

## License
Toto OS is license under the MIT License. Feel free to modify and distribute in accordance with the license.