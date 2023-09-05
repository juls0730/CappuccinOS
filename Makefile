IMAGE_NAME = CappuccinOS.iso
CARGO_OPTS = --target=src/arch/${ARCH}/${ARCH}-unknown-none.json
QEMU_OPTS = -drive format=raw,file=bin/${IMAGE_NAME}

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

.PHONY: all clean run build line-count

all: build

build: prepare-bin-files compile-bootloader compile-binaries build-iso

check: 
		cargo check

prepare-bin-files:
		rm -rf bin/iso_root
		mkdir -p bin
		mkdir -p bin/iso_root

copy-iso-files:
		# Limine files
		mkdir -p bin/iso_root/boot/limine
		mkdir -p bin/iso_root/boot/EFI/BOOT

		cp -v limine.cfg limine/limine-bios.sys bin/iso_root/boot/limine
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
		dd if=/dev/zero of=bin/${IMAGE_NAME} bs=1M count=64
		sgdisk bin/${IMAGE_NAME} -n 1:2048 -t 1:ef00
		./limine/limine bios-install bin/${IMAGE_NAME}
		mformat -i bin/${IMAGE_NAME}@@1M
		mmd -i bin/${IMAGE_NAME}@@1M ::/EFI ::/EFI/BOOT
		mcopy -i bin/${IMAGE_NAME}@@1M -s bin/iso_root/* ::/

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