#!/bin/bash
# ELF to EFI conversion script for LoongArch64
# Strip ELF header to expose embedded PE header

set -e

ELF_FILE="$1"
EFI_FILE="$2"

if [ -z "$ELF_FILE" ] || [ -z "$EFI_FILE" ]; then
    echo "Usage: $0 <input.elf> <output.efi>"
    exit 1
fi

if [ ! -f "$ELF_FILE" ]; then
    echo "Error: Input ELF file not found: $ELF_FILE"
    exit 1
fi

echo "Converting ELF to EFI: $ELF_FILE -> $EFI_FILE"
echo "Stripping ELF header to expose embedded PE header..."

# Use rust-objcopy to strip ELF header and extract raw binary
# The binary already contains PE header from our _head function
rust-objcopy --strip-all -O binary "$ELF_FILE" "$EFI_FILE"

if [ $? -eq 0 ]; then
    echo "EFI conversion completed successfully"
    echo "Output file: $EFI_FILE"
    
    # Verify the output starts with MZ signature (PE header)
    if command -v hexdump >/dev/null 2>&1; then
        echo "Checking PE header signature:"
        hexdump -C "$EFI_FILE" | head -2
    fi
else
    echo "Error: Conversion failed"
    exit 1
fi