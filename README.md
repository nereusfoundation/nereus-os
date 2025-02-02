# nereus os

A x86-64 hobby operating system written entirely from scratch in Rust.

## Getting Started

### Prerequisites
- A computer or virtual machine with UEFI support.
- A Rust toolchain installed via [rustup](https://rustup.rs/).
- QEMU (for testing)

### Cloning the Repository

Clone the repository to your local machine:

```bash
git clone https://github.com/nereusfoundation/nereus-os.git
cd nereus-os
```

### Running

To change the default configuration, use `-h` for more options.

#### QEMU

```bash
cargo run -- --release
```

#### Real Machine

```bash
sudo -E cargo run -- --run-option usb --usb /dev/<device> --release
```

