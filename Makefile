IMAGE_NAME = toto-os.iso
ARCH := ${TARGET}

ifeq (${ARCH},) 
	ARCH := x86_64
endif

.PHONY: clean run build build-32

build: prepare-bin-files prepare-iso compile-bootloader compile-kernel build-iso

prepare-bin-files:
		mkdir -p bin
		rm -f bin/${IMAGE_NAME}
		dd if=/dev/zero bs=1M count=0 seek=64 of=bin/${IMAGE_NAME}
		mkdir -p bin/iso_root

prepare-iso:
		parted -s bin/${IMAGE_NAME} mklabel gpt
		parted -s bin/${IMAGE_NAME} mkpart ESP fat32 2048s 100%
		parted -s bin/${IMAGE_NAME} set 1 esp on

build-iso:
		USED_LOOPBACK=$$(sudo losetup -Pf --show bin/${IMAGE_NAME}) && \
		sudo mkfs.fat -F 32 $${USED_LOOPBACK}p1 && \
		sudo mount $${USED_LOOPBACK}p1 bin/iso_root && \
		sudo mkdir -p bin/iso_root/EFI/BOOT && \
		sudo cp -v target/${ARCH}-unknown-none/release/toto-os.elf limine.cfg ./limine/limine-bios.sys bin/iso_root/ && \
		sudo cp -v limine/BOOT*.EFI bin/iso_root/EFI/BOOT/ && \
		sync && \
		sudo umount bin/iso_root && \
		sudo losetup -d $${USED_LOOPBACK}

compile-bootloader:
		make -C limine
		./limine/limine bios-install bin/${IMAGE_NAME}

compile-kernel:
		cargo build --release --target=./src/arch/${ARCH}/${ARCH}-unknown-none.json

run: build
		qemu-system-x86_64 -drive format=raw,file=bin/${IMAGE_NAME} -serial mon:stdio -s

clean:
		cargo clean
		rm -rf bin