#!/bin/bash
cargo test --target aarch64-unknown-none -p platform-test  --test test -- --show-output
