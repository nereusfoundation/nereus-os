# nebula uefi-loader

This project is a custom Rust UEFI (Unified Extensible Firmware Interface) bootloader, designed to load and initialize the **Nebula Kernel** on 64-bit systems. The bootloader provides the essential mechanisms to boot the Nebula Kernel in UEFI environments, adhering to modern standards while leveraging Rust's safety and performance features.

## Key Features
- **64-bit UEFI Compliant**: Ensures compatibility with modern UEFI firmware systems.
- **Rust-based Bootloader**: Written entirely in Rust to guarantee memory safety, reliability, and performance.
- **Nebula Kernel Integration**: Specifically designed to load and initialize the Nebula Kernel, with customizable options for the kernel loading process.
- **Minimalistic Design**: A lightweight and modular approach to focus on performance and simplicity.

## Getting Started

### Prerequisites

To get started, you'll need the following tools installed on your machine:

- **Rust** (latest stable or nightly): [Install Rust](https://www.rust-lang.org/tools/install)
- **QEMU**: For emulating the UEFI bootloader. Install it using your package manager:
  - On Ubuntu:
    ```bash
    sudo apt install qemu qemu-system-x86
    ```
  - On macOS:
    ```bash
    brew install qemu
    ```
- **Make**: Required to use the provided `Makefile` for building and running the project.
