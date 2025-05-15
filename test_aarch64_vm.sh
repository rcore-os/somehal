#!/bin/bash
cargo test --target aarch64-unknown-none -p platform-test --features somehal/early-debug,somehal/vm --test test -- --show-output
