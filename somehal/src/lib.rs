#![cfg_attr(not(test), no_std)]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]
#![feature(cfg_match)]
#![feature(fn_align)]
#![feature(allocator_api)]

extern crate alloc;

#[macro_use]
pub(crate) mod _alloc;

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

use mem::page::set_is_relocated;
pub use somehal_macros::entry;

pub(crate) fn to_main() -> ! {
    unsafe extern "C" {
        fn __somehal_main() -> !;
    }
    unsafe {
        set_is_relocated();
        __somehal_main();
    }
}
