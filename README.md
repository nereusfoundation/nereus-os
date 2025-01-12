# nebula os

A x86-64 hobby operating system written entirely from scratch in Rust.

## Getting Started

### Prerequisites
- A computer or virtual machine with UEFI support.
- A Rust toolchain installed via [rustup](https://rustup.rs/).
- QEMU (for testing)

### Cloning the Repository

Clone the repository to your local machine:

```bash
git clone https://github.com/nebulafoundation/nebula-os.git
cd nebula-os
```

### Running

#### QEMU

```bash
make run release=true
```

#### Real Machine

```bash
make usb USB_DEVICE=/dev/<device> release=true
```

> Makefile will later be replaced by proper boot utility


