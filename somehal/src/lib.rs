#![no_std]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]
#![feature(cfg_match)]

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
pub mod arch;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
pub mod arch;

mod archif;
pub mod console;
mod consts;
pub mod mem;
pub(crate) mod once_static;
pub(crate) mod platform;
pub(crate) mod vec;

#[cfg(use_fdt)]
pub(crate) mod fdt;

pub(crate) use archif::ArchIf;

#[cfg(feature = "early-debug")]
pub(crate) use somehal_macros::dbgln;

#[cfg(not(feature = "early-debug"))]
#[macro_export]
macro_rules! dbgln {
    ($($arg:tt)*) => {};
}
