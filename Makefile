IMAGE_NAME = toto-os.iso
ARCH := ${TARGET}
MODE := ${MODE}
CARGO_OPTS = --target=src/arch/${ARCH}/${ARCH}-unknown-none.json
QEMU_OPTS = -cdrom bin/${IMAGE_NAME}

ifeq (${MODE},)
	MODE := release
endif

ifeq (${MODE},release)
	CARGO_OPTS += --release
else
	QEMU_OPTS += -s -S
endif

ifeq (${ARCH},) 
	ARCH := x86_64
endif

.PHONY: clean run build

build: prepare-bin-files compile-bootloader compile-kernel build-iso

prepare-bin-files:
		mkdir -p bin
		mkdir -p bin/iso_root

copy-iso-files:
		cp -v target/${ARCH}-unknown-none/${MODE}/toto-os.elf limine.cfg limine/limine-bios.sys \
      limine/limine-bios-cd.bin limine/limine-uefi-cd.bin bin/iso_root/
		mkdir -p bin/iso_root/EFI/BOOT
		cp -v limine/BOOTX64.EFI bin/iso_root/EFI/BOOT/
		cp -v limine/BOOTIA32.EFI bin/iso_root/EFI/BOOT/

build-iso: copy-iso-files
		xorriso -as mkisofs -b limine-bios-cd.bin \
		        -no-emul-boot -boot-load-size 4 -boot-info-table \
		        --efi-boot limine-uefi-cd.bin \
		        -efi-boot-part --efi-boot-image --protective-msdos-label \
		        bin/iso_root -o bin/${IMAGE_NAME}
		./limine/limine bios-install bin/${IMAGE_NAME}

compile-bootloader:
		make -C limine

compile-kernel:
		cargo build ${CARGO_OPTS}

# In debug mode, open a terminal and run this command:
# gdb target/x86_64-unknown-none/debug/toto-os.elf -ex "target remote :1234"

run: build
		qemu-system-x86_64 ${QEMU_OPTS}

clean:
		cargo clean
		rm -rf bin