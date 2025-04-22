#![cfg_attr(not(test), no_std)]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]
#![feature(cfg_match)]
#![feature(fn_align)]
#![feature(allocator_api)]

extern crate alloc;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
pub mod arch;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
pub mod arch;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64/mod.rs"]
pub mod arch;

mod archif;
pub mod console;
mod entry;
pub mod mem;
pub(crate) mod once_static;
pub(crate) mod platform;

pub(crate) use archif::ArchIf;

pub use somehal_macros::entry;

unsafe extern "C" {
    pub(crate) fn __somehal_main() -> !;
}
