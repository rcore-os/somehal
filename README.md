# Somehal

## Test

```bash
cargo test --target aarch64-unknown-none -p platform-test  --test test -- --show-output
cargo test --release --target aarch64-unknown-none -p platform-test  --test test -- --show-output --uboot
cargo test --release --target riscv64gc-unknown-none-elf -p platform-test --features somehal/sv39 --features somehal/early-debug --test test -- --show-output --uboot
```