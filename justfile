# A convenient default for development: test and format
default: test format

# default + clippy; good to run before committing changes
all: default clippy

# List recipes
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

# (cargo install --locked cargo-outdated)
# Show outdated dependencies
outdated:
	cargo outdated --root-deps-only

# (cargo install --locked cargo-udeps)
# Find unused dependencies
udeps:
	cargo +nightly udeps
