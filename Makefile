ARTIFACTS_PATH = bin
IMAGE_NAME = CappuccinOS.iso
ISO_PATH = ${ARTIFACTS_PATH}/iso_root
IMAGE_PATH = ${ARTIFACTS_PATH}/${IMAGE_NAME}
CARGO_OPTS = --target=src/arch/${ARCH}/${ARCH}-unknown-none.json
QEMU_OPTS = -drive format=raw,file=${IMAGE_PATH}

ifeq (${MODE},)
	MODE := release
endif

ifeq (${MODE},release)
	CARGO_OPTS += --release
else
	QEMU_OPTS += -s -S
endif

ifneq (${UEFI},)
  RUN_OPTS := ovmf
	QEMU_OPTS += -bios bin/ovmf/OVMF.fd
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
		rm -rf ${ISO_PATH}
		mkdir -p ${ARTIFACTS_PATH}
		mkdir -p ${ISO_PATH}

copy-iso-files:
		# Limine files
		mkdir -p ${ISO_PATH}/boot/limine
		mkdir -p ${ISO_PATH}/EFI/BOOT

		cp -v limine.cfg limine/limine-bios.sys ${ISO_PATH}/boot/limine
		cp -v limine/BOOTX64.EFI ${ISO_PATH}/EFI/BOOT/

		# OS files
		cp -v target/${ARCH}-unknown-none/${MODE}/CappuccinOS.elf ${ISO_PATH}/boot

		# Application files
		mkdir -p ${ISO_PATH}/bin
		basename -s .rs src/bin/*.rs | xargs -I {} \
			cp target/${ARCH}-unknown-none/${MODE}/{}.elf ${ISO_PATH}/bin/{}

		touch ${ISO_PATH}/example.txt
		echo "Hello World" > ${ISO_PATH}/example.txt

build-iso: copy-iso-files
		rm -f ${IMAGE_PATH}
		dd if=/dev/zero of=${IMAGE_PATH} bs=1M count=0 seek=64
		sgdisk ${IMAGE_PATH} -n 1:2048 -t 1:ef00
		./limine/limine bios-install ${IMAGE_PATH}
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

run: ${RUN_OPTS} build
		qemu-system-x86_64 ${QEMU_OPTS}

line-count:
		git ls-files src | xargs wc -l

clean:
		cargo clean
		rm -rf bin
		make clean -C limine