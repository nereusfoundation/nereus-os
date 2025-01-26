# this will be replaced by the boot-cli utility later on

BOOTLOADER_DIR = uefi-loader
KERNEL_DIR = kernel 

TARGET_DIR_BOOTLOADER_DEBUG = target/x86_64-unknown-uefi/debug
TARGET_DIR_BOOTLOADER_RELEASE = target/x86_64-unknown-uefi/release

TARGET_DIR_KERNEL_DEBUG = target/x86_64-unknown-nereus/debug
TARGET_DIR_KERNEL_RELEASE = target/x86_64-unknown-nereus/release

EFI_FILE = uefi-loader.efi
KERNEL_FILE = kernel.elf
FONT_FILE = light16.psf # light16.psf / ext-light32.psf

FONT_DIR = psf
REL_TARGET_DIR = ../target

QEMU_LOG = qemu.log
STDOUT = file:stdio.log

OVMF_DIR = /usr/share/OVMF/x64
OVMF_CODE = $(OVMF_DIR)/OVMF_CODE.4m.fd
OVMF_VARS = $(OVMF_DIR)/OVMF_VARS.4m.fd

USB_DEVICE = /dev/zero

ifdef release
    CARGO_CMD = cargo build --release --target-dir=$(REL_TARGET_DIR)
    TARGET_DIR_BOOTLOADER = $(TARGET_DIR_BOOTLOADER_RELEASE)
    TARGET_DIR_KERNEL = $(TARGET_DIR_KERNEL_RELEASE)
else
    CARGO_CMD = cargo build --target-dir=$(REL_TARGET_DIR)
    TARGET_DIR_BOOTLOADER = $(TARGET_DIR_BOOTLOADER_DEBUG)
    TARGET_DIR_KERNEL = $(TARGET_DIR_KERNEL_DEBUG)
endif

.PHONY: all
all: bootloader kernel
		@echo "Build complete."

.PHONY: bootloader
bootloader:
		@echo "Building bootloader..."
		@cd $(BOOTLOADER_DIR) && $(CARGO_CMD)

.PHONY: kernel
kernel:
		@echo "Building kernel..."
		@cd $(KERNEL_DIR) && $(CARGO_CMD)

.PHONY: clippy
clippy:
		@echo "Running clippy..."
		@cd $(BOOTLOADER_DIR) && cargo clippy --target-dir=$(REL_TARGET_DIR)
		@cd $(KERNEL_DIR) && cargo clippy --target-dir=$(REL_TARGET_DIR)

.PHONY: clean
clean:
		@echo "Cleaning target directory..."
		@rm -rf target
		@echo "Cleaning build directory..."
		@rm -rf $(BUILD_DIR)
		@echo "Cleaning disk image..."
		@rm nereus-os.img
		@echo "Clean complete."

.PHONY: img
img: all
		@echo "Creating raw disk image..."
		@dd if=/dev/zero of=nereus-os.img bs=1M count=64
		@mkfs.vfat nereus-os.img
		@mkdir -p mnt
		@sudo mount nereus-os.img mnt
		@sudo mkdir -p mnt/efi/boot
		@echo "Copying UEFI file to boot directory..."
		@sudo cp $(TARGET_DIR_BOOTLOADER)/$(EFI_FILE) mnt/efi/boot/bootx64.efi
		@echo "Copying kernel file to image..."
		@sudo cp $(TARGET_DIR_KERNEL)/$(KERNEL_FILE) mnt/kernel.elf
		@echo "Copying font file to image..."
		@sudo cp $(FONT_DIR)/$(FONT_FILE) mnt/font.psf
		@sudo umount mnt
		@rmdir mnt
		@echo "Disk image nereus-os.img created successfully."

.PHONY: run
run: img 
		@echo "Running QEMU..."
		@qemu-system-x86_64 \
			-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
			-drive if=pflash,format=raw,readonly=on,file=$(OVMF_VARS) \
			-drive format=raw,file=nereus-os.img \
			-d int -D $(QEMU_LOG) -no-reboot -serial $(STDOUT) -m 512M

.PHONY: usb
usb: all
		@echo "Formatting USB drive..."
		@sudo parted $(USB_DEVICE) -- mklabel gpt
		@sudo parted $(USB_DEVICE) -- mkpart ESP fat32 1MiB 100%
		@sudo parted $(USB_DEVICE) -- set 1 esp on
		@sudo mkfs.fat -F32 $(USB_DEVICE)1
		@echo "Mounting USB drive..."
		@sudo mount $(USB_DEVICE)1 /mnt
		@echo "Creating EFI boot directory on USB drive..."
		@sudo mkdir -p /mnt/efi/boot
		@echo "Copying UEFI file to USB drive..."
		@sudo cp $(TARGET_DIR_BOOTLOADER)/$(EFI_FILE) /mnt/efi/boot/bootx64.efi
		@echo "Copying kernel file to USB drive..."
		@sudo cp $(TARGET_DIR_KERNEL)/$(KERNEL_FILE) /mnt/kernel.elf
		@echo "Copying font file to boot directory..."
		@sudo cp $(FONT_DIR)/$(FONT_FILE) /mnt/font.psf
		@echo "Unmounting USB drive..."
		@sudo umount /mnt
		@echo "USB drive is ready to boot."
