#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${HERE}/target"
TARGET_FILE="${TARGET_DIR}/ek2.tar.xz"

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

qemu-system-loongarch64 -machine virt  -cpu la464 \
     -bios target/edk2/loongarch64/code.fd \
    --nographic