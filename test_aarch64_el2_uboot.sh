#!/bin/bash
cargo test --target aarch64-unknown-none-softfloat -p test-some-rt --test test --features "hv" -- --show-output --uboot
# cargo test --target aarch64-unknown-none-softfloat -p test-some-rt --test test  -- --show-output --uboot
