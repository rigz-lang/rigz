#!/usr/bin/env bash

set -e

cargo test --workspace

wasm-pack test --node -p rigz_ast -p rigz_vm -p rigz_ast_derive -p rigz_runtime --features js --no-default-features --target wasm32-unknown-unknown

# integration tests