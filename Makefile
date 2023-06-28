IMAGE_NAME = os-image

.PHONY: clean

build: make-bin-dir compile-kernel compile-bootloader objcopy-elf-file build-os-image

make-bin-dir:
		mkdir -p bin

build-os-image:
		dd if=bin/bootloader.bin conv=notrunc of=bin/${IMAGE_NAME} bs=512
		dd if=bin/kernel.bin conv=notrunc of=bin/${IMAGE_NAME} bs=512 seek=1

compile-bootloader:
		nasm -f bin src/bootloader/bootloader.asm -o bin/bootloader.bin

objcopy-elf-file:
		objcopy -O binary target/x86_64-unknown-none/release/operating-system.elf bin/kernel.bin

compile-kernel:
		cargo build --release --target x86_64-unknown-none.json

run: build
		qemu-system-x86_64 -drive format=raw,file=bin/${IMAGE_NAME} -serial mon:stdio -s

clean:
		cargo clean
		rm -rf bin