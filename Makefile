IMAGE_NAME = toto-os.iso

.PHONY: clean run build

build: make-bin-dir compile-kernel copy-files build-os-image

make-bin-dir:
		mkdir -p bin
		mkdir -p bin/iso_root

copy-files:
		cp -v target/x86_64-unknown-none/release/toto-os.elf limine.cfg limine/limine-bios.sys \
      limine/limine-bios-cd.bin limine/limine-uefi-cd.bin bin/iso_root/
		mkdir -p bin/iso_root/EFI/BOOT
		cp -v limine/BOOT*.EFI bin/iso_root/EFI/BOOT/

build-os-image:
		xorriso -as mkisofs -b limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        --efi-boot limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        bin/iso_root -o bin/${IMAGE_NAME}
		./limine/limine bios-install bin/${IMAGE_NAME}

compile-bootloader:
		nasm -f bin src/bootloader/bootloader.asm -o bin/bootloader.bin

compile-kernel:
		cargo rustc --release --target x86_64-unknown-none.json -- -C link-arg=--script=./linker.ld

run: build
		qemu-system-x86_64 -cdrom bin/${IMAGE_NAME} -serial mon:stdio -s

clean:
		cargo clean
		rm -rf bin