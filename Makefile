IMAGE_NAME = CappuccinOS.iso
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

.PHONY: clean run build line-count

build: prepare-bin-files compile-bootloader compile-binaries build-iso

prepare-bin-files:
		rm -rf bin/iso_root
		mkdir -p bin
		mkdir -p bin/iso_root

copy-iso-files:
		# Limine files
		mkdir -p bin/iso_root/boot/limine
		mkdir -p bin/iso_root/boot/EFI/BOOT

		cp -v limine.cfg limine/limine-bios.sys \
      		limine/limine-bios-cd.bin limine/limine-uefi-cd.bin bin/iso_root/boot/limine
		cp -v limine/BOOTX64.EFI bin/iso_root/boot/EFI/BOOT/
		cp -v limine/BOOTIA32.EFI bin/iso_root/boot/EFI/BOOT/

		# OS files
		cp -v target/${ARCH}-unknown-none/${MODE}/CappuccinOS.elf bin/iso_root/boot

		# Application files
		mkdir -p bin/iso_root/bin
		basename -s .rs src/bin/*.rs | xargs -I {} \
			cp target/${ARCH}-unknown-none/${MODE}/{}.elf bin/iso_root/bin/{}

		touch bin/iso_root/example.txt
		echo "Hello World" > bin/iso_root/example.txt

build-iso: copy-iso-files
		xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
		        -no-emul-boot -boot-load-size 4 -boot-info-table \
		        --efi-boot boot/limine/limine-uefi-cd.bin \
		        -efi-boot-part --efi-boot-image --protective-msdos-label \
		        bin/iso_root -o bin/${IMAGE_NAME}
		./limine/limine bios-install bin/${IMAGE_NAME}

compile-bootloader:
		make -C limine

compile-binaries:
		cargo build ${CARGO_OPTS}

# In debug mode, open a terminal and run this command:
# gdb target/x86_64-unknown-none/debug/CappuccinOS.elf -ex "target remote :1234"

run: build
		qemu-system-x86_64 ${QEMU_OPTS}

line-count:
		git ls-files src | xargs wc -l

clean:
		cargo clean
		rm -rf bin
		make clean -C limine