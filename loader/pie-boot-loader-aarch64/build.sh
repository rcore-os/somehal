export RUSTFLAGS="-C relocation-model=pic -Clink-args=-pie"
cargo build -p pie-boot-loader-aarch64 --target aarch64-unknown-none-softfloat --release -Zbuild-std=core,alloc
rust-objcopy --strip-all -O binary target/aarch64-unknown-none-softfloat/release/pie-boot-loader-aarch64 \
 target/aarch64-unknown-none-softfloat/release/pie-boot-loader-aarch64.bin
