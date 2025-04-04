#![no_std]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]
#![feature(cfg_match)]

#[cfg_attr(target_arch = "aarch64", path = "arch/aarch64/mod.rs")]
pub mod arch;
mod archif;
pub mod console;
mod consts;
pub mod mem;
pub(crate) mod once_static;
pub(crate) mod platform;
pub(crate) mod vec;

#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
pub(crate) mod fdt;

pub(crate) use archif::ArchIf;

#[cfg(feature = "early-debug")]
pub(crate) use somehal_macros::dbgln;

#[cfg(not(feature = "early-debug"))]
#[macro_export]
macro_rules! dbgln {
    ($($arg:tt)*) => {};
}
