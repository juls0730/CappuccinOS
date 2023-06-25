.PHONY: all clean

all: cargo-build compile-bootloader copy-bootloader objcopy-elf-file build-os-image

build-os-image:
		dd if=target/x86_64-unknown-none/release/bootloader.bin conv=notrunc of=os-image bs=512
		dd if=target/x86_64-unknown-none/release/operating-system.bin conv=notrunc of=os-image bs=512 seek=1

compile-bootloader:
		nasm -f bin bootloader/bootloader.asm -o bootloader/bootloader.bin

objcopy-elf-file:
		objcopy -O binary target/x86_64-unknown-none/release/operating-system.elf target/x86_64-unknown-none/release/operating-system.bin

copy-bootloader:
		cp bootloader/bootloader.bin target/x86_64-unknown-none/release/bootloader.bin

cargo-build:
		cargo rustc --release --target x86_64-unknown-none.json

run: all
		qemu-system-x86_64 -drive format=raw,file=os-image -serial mon:stdio -s

clean:
		cargo clean
		rm -f os-image bootloader/bootloader.bin