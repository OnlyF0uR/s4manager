all: build

setup:
	rustup override set nightly

build:
	cargo build --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-pc-windows-msvc

.PHONY: all build