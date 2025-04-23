#![cfg_attr(not(all(test, target_os = "none")), no_std)]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]
#![feature(fn_align)]

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64/mod.rs"]
mod arch;

#[macro_use]
pub(crate) mod console;
#[cfg(early_uart)]
pub(crate) mod debug;

mod api;
mod archif;
mod config;
mod mem;
#[allow(unused)]
mod paging;
mod vec;

use arch::*;
use mem::boot_info;
#[cfg(early_debug)]
pub(crate) use somehal_macros::dbgln;
#[cfg(not(early_debug))]
#[macro_export]
macro_rules! dbgln {
    ($($arg:tt)*) => {};
}

pub use api::*;

pub(crate) fn relocate() {
    unsafe extern "Rust" {
        fn __vma_relocate_entry(boot_info: BootInfo);
    }

    unsafe {
        mem::save_fdt();
        __vma_relocate_entry(boot_info());
    }
}

pub fn boot_text() -> &'static [u8] {
    unsafe extern "C" {
        fn __start_boot_text();
        fn __stop_boot_text();
    }
    unsafe {
        core::slice::from_raw_parts(
            __start_boot_text as _,
            __stop_boot_text as usize - __start_boot_text as usize,
        )
    }
}

pub fn boot_data() -> &'static [u8] {
    unsafe extern "C" {
        fn __start_boot_data();
        fn __stop_boot_bss();
    }
    unsafe {
        core::slice::from_raw_parts(
            __start_boot_data as _,
            __stop_boot_bss as usize - __start_boot_data as usize,
        )
    }
}
