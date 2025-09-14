#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${HERE}/target"
TARGET_FILE="${TARGET_DIR}/ek2.tar.xz"

echo "Starting LoongArch64 kernel build and test..."

mkdir -p "${TARGET_DIR}"

if [ -f "${TARGET_FILE}" ]; then
  echo "Found ${TARGET_FILE}, skipping download."
else
  echo "${TARGET_FILE} not found. Fetching latest edk2 prebuilt release from GitHub..."

  API_URL="https://api.github.com/repos/ZR233/ovmf-prebuilt/releases/latest"

  # Prefer curl, fallback to wget
  if command -v curl >/dev/null 2>&1; then
    resp=$(curl -sSL "${API_URL}")
  elif command -v wget >/dev/null 2>&1; then
    resp=$(wget -qO- "${API_URL}")
  else
    echo "Neither curl nor wget is available. Install one to proceed." >&2
    exit 1
  fi

  # Parse JSON to find asset URL ending with .tar.xz and containing edk2 or ek2
  asset_url=$(printf "%s" "$resp" | grep -Eo 'https://[^"]+\.tar\.xz' | grep -E 'edk2|ek2' | head -n1 || true)

  if [ -z "$asset_url" ]; then
    # fallback: try to extract browser_download_url field
    asset_url=$(printf "%s" "$resp" | grep -E '"browser_download_url"' | sed -E 's/.*"(https:[^\"]+)".*/\1/' | grep -E '\.tar\.xz$' | grep -E 'edk2|ek2' | head -n1 || true)
  fi

  if [ -z "$asset_url" ]; then
    echo "Failed to find edk2 .tar.xz asset in latest release. Full API response saved to ${TARGET_DIR}/release.json" >&2
    printf "%s" "$resp" > "${TARGET_DIR}/release.json"
    exit 1
  fi

  echo "Downloading ${asset_url} -> ${TARGET_FILE}"
  if command -v curl >/dev/null 2>&1; then
    curl -L --fail -o "${TARGET_FILE}" "${asset_url}"
  else
    wget -O "${TARGET_FILE}" "${asset_url}"
  fi
fi

echo "Extracting ${TARGET_FILE} into a temporary directory and normalizing to ${TARGET_DIR}/edk2"
TMP_EXTRACT_DIR=$(mktemp -d "${TARGET_DIR}/.edk2_extract.XXXX")
tar -xJf "${TARGET_FILE}" -C "${TMP_EXTRACT_DIR}"

# If archive contains a single top-level directory, move/rename it to target/edk2.
# Otherwise, create target/edk2 and move all extracted entries into it.
shopt -s nullglob
entries=("${TMP_EXTRACT_DIR}"/*)
if [ ${#entries[@]} -eq 1 ] && [ -d "${entries[0]}" ]; then
  echo "Single top-level directory found: $(basename "${entries[0]}")"
  rm -rf "${TARGET_DIR}/edk2"
  mv "${entries[0]}" "${TARGET_DIR}/edk2"
else
  echo "Multiple top-level entries or files found, moving contents into ${TARGET_DIR}/edk2"
  rm -rf "${TARGET_DIR}/edk2"
  mkdir -p "${TARGET_DIR}/edk2"
  mv "${TMP_EXTRACT_DIR}"/* "${TARGET_DIR}/edk2/" || true
fi
shopt -u nullglob
rm -rf "${TMP_EXTRACT_DIR}"

echo "Extraction complete: ${TARGET_DIR}/edk2"

echo "Building LoongArch64 kernel..."

# Build the somehal kernel for LoongArch64
echo "Compiling somehal for loongarch64-unknown-none-softfloat target..."
cargo build --target loongarch64-unknown-none-softfloat --release -p somehal

# Check if the build was successful
if [ ! -f "${TARGET_DIR}/loongarch64-unknown-none-softfloat/release/libsomehal.rlib" ]; then
    echo "Failed to build somehal library"
    exit 1
fi

echo "Building test binary..."
cargo test --target loongarch64-unknown-none-softfloat -p test-some-rt --test test --no-run -- --show-output

# Find the built kernel binary 
KERNEL_ELF=$(find "${TARGET_DIR}/loongarch64-unknown-none-softfloat" -name "test-*" -type f -executable | head -n1)

if [ -z "$KERNEL_ELF" ] || [ ! -f "$KERNEL_ELF" ]; then
    echo "Error: Could not find built kernel ELF file"
    echo "Looking in: ${TARGET_DIR}/loongarch64-unknown-none-softfloat"
    find "${TARGET_DIR}/loongarch64-unknown-none-softfloat" -name "*test*" -type f
    exit 1
fi

echo "Found kernel ELF: $KERNEL_ELF"

# Copy kernel to a known location
cp "$KERNEL_ELF" "${TARGET_DIR}/kernel.elf"

echo "Kernel build completed successfully!"
echo "Starting QEMU with EFI firmware and LoongArch64 kernel..."

# Check if qemu-system-loongarch64 is available
if ! command -v qemu-system-loongarch64 >/dev/null 2>&1; then
    echo "Error: qemu-system-loongarch64 not found. Please install QEMU with LoongArch64 support."
    exit 1
fi

# Check if EFI firmware exists
if [ ! -f "${TARGET_DIR}/edk2/loongarch64/code.fd" ]; then
    echo "Error: LoongArch64 EFI firmware not found at ${TARGET_DIR}/edk2/loongarch64/code.fd"
    ls -la "${TARGET_DIR}/edk2/" || true
    ls -la "${TARGET_DIR}/edk2/loongarch64/" || true
    exit 1
fi

echo "Running QEMU with the following configuration:"
echo "  Machine: virt"
echo "  CPU: la464"
echo "  EFI firmware: ${TARGET_DIR}/edk2/loongarch64/code.fd"
echo "  Kernel: ${TARGET_DIR}/kernel.elf"
echo ""

# Run QEMU with the LoongArch64 EFI firmware and our kernel
qemu-system-loongarch64 \
    -machine virt \
    -cpu la464 \
    -m 1G \
    -bios "${TARGET_DIR}/edk2/loongarch64/code.fd" \
    -kernel "${TARGET_DIR}/kernel.elf" \
    -nographic \
    -serial stdio \
    -monitor none \
    -d guest_errors \
    "$@"

