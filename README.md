# Toto OS
Toto OS is a small operating system written in rust. This README will guide you through the process of building and running Toto OS.

## Prerequisites
Before building Toto OS, make sure you have th following installed on your machine:

- rust (nightly)
- cargo

If you are interested in running Toto OS with the `make run` command, you will need to install QEMU.

## Getting started
To build Toto OS, follow these steps:

1. Clone the repository.
	 ```BASH
	 git clone https://github.com/juls0730/toto-os.git
	 cd toto-os
	 ```
2. Build Toto OS using `make` command:
	 ```BASH
	 make build
	 ```

You will be able to find the built image in `bin/os-image`.

<br/>

If you instead want to build Toto OS & run it with qemu, you can simply run make like so:

```BASH
make run
```

## License
Toto OS is license under the MIT License. Feel free to modify and distribute in accordance with the license.