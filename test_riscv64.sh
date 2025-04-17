#!/bin/bash
cargo test --target riscv64gc-unknown-none-elf -p platform-test --features somehal/early-debug --features somehal/sv39 --test test -- --show-output
