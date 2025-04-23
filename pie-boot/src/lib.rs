#![cfg_attr(not(all(test, target_os = "none")), no_std)]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]
#![feature(cfg_match)]
#![feature(fn_align)]
#![feature(pointer_is_aligned_to)]

use core::ptr::NonNull;

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
mod mem;
#[allow(unused)]
mod paging;
mod vec;
mod config;

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
        __vma_relocate_entry(boot_info());
    }
}
