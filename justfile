# List recipes
list:
	@just --list --unsorted

# A convenient recipe for development: test and format
dev: test format

# dev + clippy; good to run before committing changes
all: dev clippy

# cargo watch
watch *cmd='check':
    cargo watch -c -x '{{ cmd }}'

# cargo check
check:
    cargo check

# cargo clean
clean:
    cargo clean

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

# List git tags
tags:
  git tag -l | sort -V | tail

# Create and push git tag (calls tq)
tag-and-push:
  #!/usr/bin/env bash
  version=$(tq -f Cargo.toml 'package.version')
  echo "tagging and pushing v${version}"
  git tag v${version}
  git push origin v${version}

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
