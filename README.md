# nereus os

A x86-64 hobby operating system written entirely from scratch in Rust.

## Getting Started

### Prerequisites
- Nix package manager

### Cloning the Repository

Clone the repository to your local machine:

```bash
git clone https://github.com/nereusfoundation/nereus-os.git
cd nereus-os
```

The nix flake provides several outputs to run nereusOS.
The `kernel` and `uefi-loader` can be built using:
```bash
nix build .#kernel
```
> for loader: .#loader

### Running

#### QEMU

```bash
nix run .
```

#### Real Machine

```bash
nix run .#flash
```

## Development

In order to work on the project, the flake provides a dev shell, which can be invoked with:
```bash
nix develop
```
