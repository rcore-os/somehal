#!/bin/bash
cargo test --target riscv64gc-unknown-none-elf -p platform-test --features somehal/early-debug --test test -- --show-output
