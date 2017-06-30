${HOME}/.cargo/bin/cargo-fmt:
	cargo install rustfmt --vers 0.8.3

fmt: ${HOME}/.cargo/bin/cargo-fmt
	PATH=${HOME}/.cargo/bin:${PATH} cargo fmt -- --write-mode=diff

build:
	RUSTFLAGS='-D warnings' cargo build

test:
	RUSTFLAGS='-D warnings' RUST_BACKTRACE=1 cargo test -- --skip test_basics

sudo_test:
	sudo env "PATH=${PATH}" RUSTFLAGS='-D warnings' RUST_BACKTRACE=1 cargo test

clippy:
	RUSTFLAGS='-D warnings' cargo build --features "clippy" --verbose

.PHONY:
	fmt
	build
	test
	sudo_test
	clippy
