ARTIFACTS_PATH ?= bin
IMAGE_NAME ?= CappuccinOS.iso
ISO_PARTITION_TYPE ?= GPT
MODE ?= release
ARCH ?= x86_64

ISO_PATH = ${ARTIFACTS_PATH}/iso_root
INITRAMFS_PATH = ${ARTIFACTS_PATH}/initramfs
IMAGE_PATH = ${ARTIFACTS_PATH}/${IMAGE_NAME}
CARGO_OPTS = --target=src/arch/${ARCH}/${ARCH}-unknown-none.json
QEMU_OPTS = -m 512M -drive format=raw,file=${IMAGE_PATH}

ifeq (${MODE},release)
	CARGO_OPTS += --release
else
	QEMU_OPTS += -s -S
endif

ifneq (${UEFI},)
	RUN_OPTS := ovmf
	QEMU_OPTS += -bios bin/ovmf/OVMF.fd
endif

.PHONY: all check prepare-bin-files copy-initramfs-files compile-initramfs copy-iso-files build-iso compile-bootloader compile-binaries ovmf clean run build line-count

all: build

build: prepare-bin-files compile-bootloader compile-binaries compile-initramfs build-iso

check: 
		cargo check

prepare-bin-files:
		# Remove ISO and everything in the bin directory
		rm -f ${IMAGE_PATH}
		rm -rf ${ARTIFACTS_PATH}/*

		# Make bin/ bin/iso_root and bin/initramfs
		mkdir -p ${ARTIFACTS_PATH}
		mkdir -p ${ISO_PATH}
		mkdir -p ${INITRAMFS_PATH}

copy-initramfs-files:
		# Stub for now ;)
		touch ${INITRAMFS_PATH}/example.txt
		echo "Hello World from Initramfs" > ${INITRAMFS_PATH}/example.txt

compile-initramfs: copy-initramfs-files
		mksquashfs ${INITRAMFS_PATH} ${ARTIFACTS_PATH}/initramfs.img

copy-iso-files:
		# Limine files
		mkdir -p ${ISO_PATH}/boot/limine
		mkdir -p ${ISO_PATH}/EFI/BOOT

		cp -v limine.cfg limine/limine-bios.sys ${ISO_PATH}/boot/limine
		cp -v limine/BOOTX64.EFI ${ISO_PATH}/EFI/BOOT/

		# OS files
		cp -v target/${ARCH}-unknown-none/${MODE}/CappuccinOS.elf ${ISO_PATH}/boot
		cp -v ${ARTIFACTS_PATH}/initramfs.img ${ISO_PATH}/boot

		# Application files
		mkdir -p ${ISO_PATH}/bin
		basename -s .rs src/bin/*.rs | xargs -I {} \
			cp target/${ARCH}-unknown-none/${MODE}/{}.elf ${ISO_PATH}/bin/{}

		touch ${ISO_PATH}/example.txt
		echo "Hello World from the hard drive" > ${ISO_PATH}/example.txt

partition-iso: copy-iso-files
		# Make empty ISO of 64M in size
		dd if=/dev/zero of=${IMAGE_PATH} bs=1M count=0 seek=64
ifeq (${ISO_PARTITION_TYPE},GPT)
		# Make ISO a GPT disk with 1 partition starting at sector 2048 that is 32768 sectors, or 16MiB, in size
		# Then a second partition spanning the rest of the disk
		sgdisk ${IMAGE_PATH} -n 1:2048:+32768 -t 1:ef00 -n 2
else
		# Make ISO a MBR disk with 1 partition starting at sector 2048 that is 32768 sectors, or 16MiB, in size
		# Then a second partition spanning the rest of the disk
		parted -a none ${IMAGE_PATH} mklabel msdos
		parted -a none ${IMAGE_PATH} mkpart primary 2048s 34815s
		parted -a none ${IMAGE_PATH} mkpart primary 34816s 100%
		parted -a none ${IMAGE_PATH} set 1 boot on
endif

build-iso: partition-iso
		# Install the Limine bootloader on the ISO
		./limine/limine bios-install ${IMAGE_PATH}

		# Make a FAT32 FS and copy files in /bin/iso_root into the ISO starting at 1M or exactly 2048 sectors
		mformat -F -i ${IMAGE_PATH}@@1M
		mmd -i ${IMAGE_PATH}@@1M ::/EFI ::/EFI/BOOT
		mcopy -i ${IMAGE_PATH}@@1M -s ${ISO_PATH}/* ::/

compile-bootloader:
		make -C limine

compile-binaries:
		cargo build ${CARGO_OPTS}

ovmf:
	mkdir -p bin/ovmf
	cd bin/ovmf && curl -Lo OVMF.fd https://retrage.github.io/edk2-nightly/bin/RELEASEX64_OVMF.fd

# In debug mode, open a terminal and run this command:
# gdb target/x86_64-unknown-none/debug/CappuccinOS.elf -ex "target remote :1234"

run: build ${RUN_OPTS}
		qemu-system-x86_64 ${QEMU_OPTS}

line-count:
		cloc --quiet --exclude-dir=bin --csv src/ | tail -n 1 | awk -F, '{print $$5}'
clean:
		cargo clean
		rm -rf bin
		make clean -C limine