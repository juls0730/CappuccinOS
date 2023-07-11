IMAGE_NAME = toto-os.iso
ARCH := ${TARGET}
MODE := ${MODE}
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

.PHONY: clean run build

build: prepare-bin-files compile-bootloader compile-kernel build-iso

prepare-bin-files:
		mkdir -p bin
		rm -f bin/${IMAGE_NAME}
		dd if=/dev/zero bs=1M count=0 seek=64 of=bin/${IMAGE_NAME}
		mkdir -p bin/iso_root

prepare-iso:
		parted -s bin/${IMAGE_NAME} mklabel gpt
		parted -s bin/${IMAGE_NAME} mkpart ESP fat32 2048s 100%
		parted -s bin/${IMAGE_NAME} set 1 esp on
		sudo losetup -Pf --show bin/${IMAGE_NAME} > bin/used_loopback
		./limine/limine bios-install bin/${IMAGE_NAME}

mount-iso:
		sudo mkfs.fat -F 32 ${shell cat bin/used_loopback}p1
		sudo mount ${shell cat bin/used_loopback}p1 bin/iso_root

copy-files:
		sudo mkdir -p bin/iso_root/EFI/BOOT
		sudo cp -v target/${ARCH}-unknown-none/${MODE}/toto-os.elf limine.cfg ./limine/limine-bios.sys bin/iso_root/
		sudo cp -v limine/BOOT*.EFI bin/iso_root/EFI/BOOT/

unmount-iso:
		sync
		sudo umount bin/iso_root
		sudo losetup -d ${shell cat bin/used_loopback}
		rm -rf bin/used_loopback

build-iso: prepare-iso mount-iso copy-files unmount-iso

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