.PHONY: b br fmt lint

b: fmt
	cargo +nightly build

br: fmt
	cargo +nightly build --release

fmt:
	cargo +nightly fmt

lint:
	cargo +nightly clippy
