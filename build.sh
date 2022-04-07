#!/usr/bin/env bash

cargo test
cargo clean -p "$(cargo read-manifest | jq -r '.name')"
cargo clippy -- -D warnings
cargo fmt -- --check
