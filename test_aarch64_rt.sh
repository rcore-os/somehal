#!/bin/bash
cargo test --target aarch64-unknown-none -p test-some-rt  --test test -- --show-output
