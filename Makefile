ARTIFACTS_PATH ?= bin
IMAGE_NAME ?= CappuccinOS.iso
ISO_PARTITION_TYPE ?= GPT
MODE ?= release
ARCH ?= x86_64
MEMORY ?= 512M
QEMU_OPTS ?= 
MKSQUASHFS_OPTS ?= 

ISO_PATH = ${ARTIFACTS_PATH}/iso_root
INITRAMFS_PATH = ${ARTIFACTS_PATH}/initramfs
IMAGE_PATH = ${ARTIFACTS_PATH}/${IMAGE_NAME}
CARGO_OPTS = --target=src/arch/${ARCH}/${ARCH}-unknown-none.json
QEMU_OPTS += -m ${MEMORY} -drive id=hd0,format=raw,file=${IMAGE_PATH}
LIMINE_BOOT_VARIATION = X64

ifeq (${MODE},release)
	CARGO_OPTS += --release
else
	QEMU_OPTS += -s -S
endif

ifeq (${ARCH},riscv64)
	LIMINE_BOOT_VARIATION := RISCV64
	UEFI := true
endif

ifneq (${UEFI},)
	RUN_OPTS := ovmf-${ARCH}
	ifeq (${ARCH},riscv64)
		QEMU_OPTS += -drive if=pflash,unit=0,format=raw,file=ovmf/ovmf-riscv64/OVMF.fd -M virt
	else
		QEMU_OPTS += -bios ovmf/ovmf-${ARCH}/OVMF.fd
	endif
endif

.PHONY: all check run-scripts prepare-bin-files copy-initramfs-files compile-initramfs copy-iso-files build-iso compile-bootloader compile-binaries ovmf clean run build line-count

all: build

build: prepare-bin-files compile-bootloader compile-binaries run-scripts compile-initramfs build-iso

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
		echo "Hello World from Initramfs" > ${INITRAMFS_PATH}/example.txt
		echo "Second file for testing" > ${INITRAMFS_PATH}/example2.txt
		mkdir -p ${INITRAMFS_PATH}/firstdir/seconddirbutlonger/
		echo "Hell yeah, we getting a working initramfs using a custom squashfs driver!!" > ${INITRAMFS_PATH}/firstdir/seconddirbutlonger/yeah.txt

compile-initramfs: copy-initramfs-files
		# Make squashfs without compression temporaily so I can get it working before I have to write a gzip driver
		mksquashfs ${INITRAMFS_PATH} ${ARTIFACTS_PATH}/initramfs.img ${MKSQUASHFS_OPTS}

run-scripts:
		nm target/${ARCH}-unknown-none/${MODE}/CappuccinOS.elf > scripts/symbols.table
		@if [ ! -d "scripts/rustc_demangle" ]; then \
			echo "Cloning rustc_demangle.py into scripts/rustc_demangle/..."; \
			git clone "https://github.com/juls0730/rustc_demangle.py" "scripts/rustc_demangle"; \
		else \
			echo "Folder scripts/rustc_demangle already exists. Skipping clone."; \
		fi
		python scripts/demangle-symbols.py
		mv scripts/symbols.table ${INITRAMFS_PATH}/

		python scripts/font.py
		mv scripts/font.psf ${INITRAMFS_PATH}/

copy-iso-files:
		# Limine files
		mkdir -p ${ISO_PATH}/boot/limine
		mkdir -p ${ISO_PATH}/EFI/BOOT

		cp -v limine.cfg limine/limine-bios.sys ${ISO_PATH}/boot/limine
		cp -v limine/BOOT${LIMINE_BOOT_VARIATION}.EFI ${ISO_PATH}/EFI/BOOT/

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

build-iso: partition-iso copy-initramfs-files
ifeq (${ARCH},x86_64)
		# Install the Limine bootloader for bios installs
		./limine/limine bios-install ${IMAGE_PATH}
endif

		# Make a FAT32 FS and copy files in /bin/iso_root into the ISO starting at 1M or exactly 2048 sectors
		mformat -F -i ${IMAGE_PATH}@@1M
		mmd -i ${IMAGE_PATH}@@1M ::/EFI ::/EFI/BOOT
		mcopy -i ${IMAGE_PATH}@@1M -s ${ISO_PATH}/* ::/

compile-bootloader:
	@if [ ! -d "limine" ]; then \
		echo "Cloning Limine into limine/..."; \
		git clone https://github.com/limine-bootloader/limine.git --branch=v5.x-branch-binary --depth=1; \
	else \
		echo "Folder limine already exists. Skipping clone."; \
	fi
		make -C limine

compile-binaries:
		cargo build ${CARGO_OPTS}

ovmf-x86_64: ovmf
	mkdir -p ovmf/ovmf-x86_64
	@if [ ! -d "ovmf/ovmf-x86_64/OVMF.fd" ]; then \
		cd ovmf/ovmf-x86_64 && curl -Lo OVMF.fd https://retrage.github.io/edk2-nightly/bin/RELEASEX64_OVMF.fd; \
	fi

ovmf-riscv64: ovmf
	mkdir -p ovmf/ovmf-riscv64
	@if [ ! -d "ovmf/ovmf-riscv64/OVMF.fd" ]; then \
		cd ovmf/ovmf-riscv64 && curl -o OVMF.fd https://retrage.github.io/edk2-nightly/bin/RELEASERISCV64_VIRT_CODE.fd && dd if=/dev/zero of=OVMF.fd bs=1 count=0 seek=33554432; \
	fi

# In debug mode, open a terminal and run this command:
# gdb target/x86_64-unknown-none/debug/CappuccinOS.elf -ex "target remote :1234"

run: build ${RUN_OPTS} run-${ARCH}

run-x86_64:
	qemu-system-x86_64 ${QEMU_OPTS}

run-riscv64:
	qemu-system-riscv64 ${QEMU_OPTS} -M virt -cpu rv64 -device ramfb -device qemu-xhci -device usb-kbd -device virtio-scsi-pci,id=scsi -device scsi-hd,drive=hd0

line-count:
		cloc --quiet --exclude-dir=bin --csv src/ | tail -n 1 | awk -F, '{print $$5}'
clean:
		cargo clean
		rm -rf bin
		make clean -C limine