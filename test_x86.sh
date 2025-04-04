#!/bin/bash
cargo test --target x86_64-unknown-none -p platform-test --features somehal/early-debug --test test -- --show-output
