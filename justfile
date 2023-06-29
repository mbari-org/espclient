# List recipes
list:
	@just --list --unsorted

# A convenient recipe for development: test and format
dev: test format

# dev + clippy; good to run before committing changes
all: dev clippy

# Run tests
test:
	cargo test

# Run espclient (e.g.:  just run --help)
run *args='':
	cargo run -- {{ args }}

# Format source code
format:
	cargo fmt

# Run clippy
clippy:
	cargo clippy --all-targets -- -D warnings

# Build release
release:
	cargo build --release

# Install locally
install: release
	cargo install --path .

# (cargo install --locked cargo-outdated)
# Show outdated dependencies
outdated:
	cargo outdated --root-deps-only

# (cargo install --locked cargo-udeps)
# Find unused dependencies
udeps:
	cargo +nightly udeps

# cargo update
update:
	cargo update
