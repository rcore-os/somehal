# SomeHAL

[![Crates.io](https://img.shields.io/crates/v/somehal.svg)](https://crates.io/crates/somehal)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)

SomeHAL is a hardware abstraction layer library designed specifically for AArch64 chips with MMU functionality. It provides bootloader capabilities to complete MMU initialization and run code at specified virtual addresses.

## Features

- 🚀 **Dynamic MMU Initialization**: Automatically configures page tables, enables MMU, and redirects to virtual addresses
- 📱 **Multi-Privilege Level Support**: Supports both EL1 and EL2 (hypervisor) privilege levels
- 🔧 **Position Independent Boot**: Uses PIE (Position Independent Executable) bootloader
- 💾 **Memory Management**: Automatically parses device tree and manages memory regions
- 🐛 **Early Debug Support**: Provides early debug output functionality
- ⚡ **Multi-Core Support**: Supports multi-core CPU boot

## Architecture Support

- **AArch64**: ✅ Full support
  - EL1 (Exception Level 1)
  - EL2 (Exception Level 2, Hypervisor)

## Getting Started

### Adding Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
somehal = "0.3"

```

### Basic Usage

```rust
#![no_std]
#![no_main]

use somehal::{entry, BootInfo};

#[entry]
fn main(boot_info: &BootInfo) -> ! {
    // Your kernel main function
    println!("Hello from virtual address!");
    
    // Access boot information
    println!("FDT address: {:p}", boot_info.fdt);
    println!("Memory regions: {}", boot_info.memory_regions.len());
    
    // Your kernel logic
    loop {}
}
```

### Multi-Core Support

```rust
use somehal::{entry, secondary_entry, BootInfo};

#[entry]
fn main(boot_info: &BootInfo) -> ! {
    // Primary CPU startup logic
    println!("Primary CPU started");
    
    // Start other cores
    let secondary_addr = somehal::secondary_entry_addr();
    // Start other cores through your method...
    
    loop {}
}

#[secondary_entry]
fn secondary_main(cpu_id: usize) -> ! {
    // Secondary CPU startup logic
    println!("Secondary CPU {} started", cpu_id);
    loop {}
}
```

## Core Concepts

### Boot Process

1. **Kernel Entry** (`_start`) - Save boot parameters
2. **PIE Bootloader** - Position-independent bootloader responsible for:
   - Parsing device tree (FDT)
   - Configuring page table mappings
   - Enabling MMU
   - Redirecting to virtual addresses
3. **Virtual Entry** - Call user's main function

### Memory Layout

```text
Virtual Address Space:
0xffff_0000_0000_0000  +----------------+
                       |   Kernel Space |
                       |                |
0xffff_8000_0000_0000  +----------------+
                       |  Linear Mapping|
                       | (Physical Mem) |
0xffff_0000_0000_0000  +----------------+
```

### BootInfo Structure

`BootInfo` provides important boot-time information:

```rust
pub struct BootInfo {
    pub fdt: Option<NonNull<u8>>,           // Device tree pointer
    pub memory_regions: MemoryRegions,      // Memory region list
    pub kimage_start_lma: u64,              // Kernel load address
    pub kimage_start_vma: u64,              // Kernel virtual address
    pub free_memory_start: *mut u8,         // Free memory start address
    pub pg_start: u64,                      // Page table start address
    pub cpu_id: usize,                      // CPU ID
}
```

## Feature Flags

- `hv`: Enable hypervisor mode (EL2) support

### Hypervisor Mode Example

```toml
[dependencies]
somehal = { version = "0.3", features = ["hv"] }
```

## Memory Management

### Getting Memory Region Information

```rust
use somehal::{BootInfo, MemoryRegionKind};

#[entry]
fn main(boot_info: &BootInfo) -> ! {
    for region in boot_info.memory_regions.iter() {
        match region.kind {
            MemoryRegionKind::Ram => {
                println!("RAM: {:#x} - {:#x}", region.start, region.end);
            }
            MemoryRegionKind::Reserved => {
                println!("Reserved: {:#x} - {:#x}", region.start, region.end);
            }
        }
    }
    
    loop {}
}
```

### I/O Memory Mapping

```rust
use somehal::mem::iomap;

// Map I/O device memory
let device_base = 0x1000_0000;
let device_size = 0x1000;

match iomap(device_base, device_size) {
    Ok(virt_addr) => {
        println!("Device mapped at {:p}", virt_addr);
        // Use virtual address to access device
    }
    Err(e) => {
        println!("Failed to map device: {:?}", e);
    }
}
```

## Debug Features

### Early Debug Output

```rust
use somehal::*;

println!("Early boot message");
```

## Build Configuration

### Linker Script

SomeHAL requires specific linker script configuration. Create `link.ld` in your project:

```ld
STACK_SIZE = 0x40000;

INCLUDE "somehal.x"

SECTIONS
{
   # other sections...
}
```

### Build Script

In `build.rs`:

```rust
fn main() {
    // SomeHAL will automatically configure necessary linking parameters
    println!("cargo:rustc-link-arg=-Tlink.ld");
}
```

## Platform Testing

The project provides test configurations for multiple platforms:

```bash
# Test runtime
cargo test --target aarch64-unknown-none-softfloat -p test-some-rt

# Test configuration with VM functionality
cargo test --target aarch64-unknown-none-softfloat -p test-some-rt --features hv
```

## Example Projects

Check the `tests/test-some-rt` directory for complete usage examples.

## Important Notes

1. **Target Architecture**: Currently only supports `aarch64-unknown-none-softfloat` target
2. **no_std**: This is a no_std library for bare-metal environments
3. **MMU Requirement**: Requires hardware MMU support
4. **Device Tree**: Requires a valid device tree (FDT) to obtain hardware information

## License

This project is licensed under the MIT License. See the [LICENSE](../LICENSE) file for details.

## Contributing

Issues and pull requests are welcome!
