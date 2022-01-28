# A convenient default for development: test and format
default: test format

# default + clippy; good to run before committing changes
all: default clippy

# List recipes (needs `just`)
list:
	@just --list --unsorted

# Run tests
test:
	cargo test

# Format source code
format:
	cargo fmt -- --check

# Run clippy
clippy:
	cargo clippy --all-targets -- -D warnings

# Build release
release:
	cargo build --release

# Install locally
install:
	cargo install --path .
