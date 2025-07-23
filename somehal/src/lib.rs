#![cfg_attr(target_os = "none", no_std)]
#![cfg(target_os = "none")]

extern crate alloc;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;

pub use arch::*;

mod common;
pub mod early_debug;
mod lazy_static;
mod loader;

pub use common::entry::boot_info;
pub use kdef_pgtable::{KIMAGE_VADDR, KIMAGE_VSIZE, KLINER_OFFSET};
use pie_boot_if::EarlyBootArgs;
pub use pie_boot_if::{BootInfo, MemoryRegion, MemoryRegionKind, MemoryRegions};
#[allow(unused)]
use pie_boot_macros::start_code;
pub use pie_boot_macros::{entry, irq_handler, secondary_entry};

#[allow(unused)]
#[unsafe(link_section = ".data")]
static mut BOOT_ARGS: EarlyBootArgs = EarlyBootArgs::new();

#[unsafe(link_section = ".data")]
static mut BOOT_PT: usize = 0;

/// secondary entry address
/// arg0 is stack top
pub fn secondary_entry_addr() -> usize {
    let ptr = arch::_start_secondary as usize;
    ptr - boot_info().kcode_offset()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn __pie_boot_default_secondary(_cpu_id: usize) {}
