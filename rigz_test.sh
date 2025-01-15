#!/usr/bin/env bash

set -e

cargo run -- test rigz_tests

cargo run --no-default-features --features js -- test rigz_tests