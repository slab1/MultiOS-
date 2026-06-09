.PHONY: all build check test clean qemu-x86_64 qemu-arm64 fmt clippy

# Default target
all: check build

# Build targets
build:
	cargo build --target x86_64-unknown-none --release

build-x86_64:
	cargo build --target x86_64-unknown-none --release

build-arm64:
	cargo build --target aarch64-unknown-none --release

build-riscv64:
	cargo build --target riscv64gc-unknown-none-elf --release

# Check (faster than full build)
check:
	cargo check --target x86_64-unknown-none

check-all:
	cargo check --target x86_64-unknown-none
	cargo check --target aarch64-unknown-none
	cargo check --target riscv64gc-unknown-none-elf

# Run tests (unit tests on host target)
test:
	cargo test --lib

# Clean build artifacts
clean:
	cargo clean

# QEMU testing
qemu-x86_64: build-x86_64
	qemu-system-x86_64 -cdrom target/x86_64-unknown-none/release/multios.iso -m 512M

qemu-arm64: build-arm64
	qemu-system-aarch64 -cdrom target/aarch64-unknown-none/release/multios.iso -m 512M

# Code quality
fmt:
	cargo fmt --check

clippy:
	cargo clippy --target x86_64-unknown-none -- -D warnings

# Build documentation
doc:
	cargo doc --no-deps

# Setup development environment
setup:
	rustup target add x86_64-unknown-none
	rustup target add aarch64-unknown-none
	rustup target add riscv64gc-unknown-none-elf
	rustup component add rust-src
	rustup component add llvm-tools
	cargo install bootimage

# Create a bootable image
image:
	cargo bootimage --target x86_64-unknown-none

help:
	@echo "MultiOS Build System"
	@echo "===================="
	@echo "make build         - Build for x86_64"
	@echo "make check         - Check compilation"
	@echo "make test          - Run unit tests"
	@echo "make qemu-x86_64   - Run in QEMU (x86_64)"
	@echo "make fmt           - Check code formatting"
	@echo "make clippy        - Run clippy lints"
	@echo "make setup         - Install dev dependencies"
	@echo "make image         - Create bootable image"
