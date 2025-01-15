#!/usr/bin/env bash

set -e

cargo test --workspace

cargo run -- test rigz_tests

wasm-pack test --node -p rigz_ast -p rigz_vm -p rigz_ast_derive -p rigz_runtime --features js --no-default-features

cargo run --no-default-features --features js -- test rigz_tests

# integration tests