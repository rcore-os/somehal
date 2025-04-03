#!/bin/bash
cargo test --release --target aarch64-unknown-none -p platform-test  --test test -- --show-output --uboot

