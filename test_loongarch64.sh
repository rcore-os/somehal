#!/usr/bin/env bash
set -euo pipefail


HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${HERE}/target"
TARGET_FILE="${TARGET_DIR}/ek2.tar.xz"

# 检查是否传入 debug 参数
DEBUG_MODE=0
if [[ $# -ge 1 && "$1" == "debug" ]]; then
  DEBUG_MODE=1
  echo "[DEBUG] QEMU 将以调试模式启动，监听 1234 端口..."
fi

echo "Starting LoongArch64 kernel build and test..."

mkdir -p "${TARGET_DIR}"

if [ -f "${TARGET_FILE}" ]; then
  echo "Found ${TARGET_FILE}, skipping download."
else
  echo "${TARGET_FILE} not found. Fetching latest edk2 prebuilt release from Gitee (gitee.com/zr233/ovmf-prebuilt)..."

  API_URL="https://gitee.com/api/v5/repos/zr233/ovmf-prebuilt/releases/latest"

  # Prefer curl, fallback to wget
  if command -v curl >/dev/null 2>&1; then
    resp=$(curl -sSL "${API_URL}")
  elif command -v wget >/dev/null 2>&1; then
    resp=$(wget -qO- "${API_URL}")
  else
    echo "Neither curl nor wget is available. Install one to proceed." >&2
    exit 1
  fi

  # Try to use jq if available for robust parsing
  asset_url=""
  if command -v jq >/dev/null 2>&1; then
    # Gitee release JSON has assets array; each asset may have browser_download_url or direct url
    asset_url=$(printf "%s" "$resp" | jq -r '.assets[]?.browser_download_url // .assets[]?.url // empty' | grep -E '\.tar\.xz$' | grep -E 'edk2|ek2' | head -n1 || true)
  else
    # Fallback parsing: look for any https URL ending with .tar.xz
    asset_url=$(printf "%s" "$resp" | grep -Eo 'https://[^"]+\.tar\.xz' | grep -E 'edk2|ek2' | head -n1 || true)
    if [ -z "$asset_url" ]; then
      asset_url=$(printf "%s" "$resp" | grep -E '"browser_download_url"' | sed -E 's/.*"(https:[^\"]+)".*/\1/' | grep -E '\.tar\.xz$' | grep -E 'edk2|ek2' | head -n1 || true)
    fi
  fi

  if [ -z "$asset_url" ]; then
    echo "Failed to find edk2 .tar.xz asset in latest Gitee release. Full API response saved to ${TARGET_DIR}/release.json" >&2
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

echo "Building test binary..."
cargo test --target loongarch64-unknown-none-softfloat -p test-some-rt --test test --no-run  -- --show-output

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

# Convert ELF to EFI format by stripping ELF header
echo "Converting ELF to EFI format..."
KERNEL_EFI="${TARGET_DIR}/kernel.efi"
./elf2efi.sh "$KERNEL_ELF" "$KERNEL_EFI"

if [ ! -f "$KERNEL_EFI" ]; then
    echo "Error: EFI conversion failed"
    exit 1
fi

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
echo "  Kernel: ${KERNEL_EFI}"
echo ""

# KERNEL_EFI="/home/zhourui/opensource/Build/LoongArchVirtQemu/DEBUG_GCC5/LOONGARCH64/HelloWorld.efi"

echo "Trying direct kernel boot method..."
QEMU_CMD=(qemu-system-loongarch64 \
    -machine virt \
    -cpu la464 \
    -m 1G \
    -bios "${TARGET_DIR}/edk2/loongarch64/code.fd" \
    -kernel "${KERNEL_EFI}" \
    -nographic \
    -serial stdio \
    -monitor none \
    -d guest_errors)

if [ "$DEBUG_MODE" -eq 1 ]; then
  QEMU_CMD+=( -S -gdb tcp::1234 )
fi

"${QEMU_CMD[@]}"


