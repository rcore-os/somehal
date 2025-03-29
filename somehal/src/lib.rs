#![no_std]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]

#[cfg_attr(target_arch = "aarch64", path = "arch/aarch64/mod.rs")]
pub mod arch;
mod archif;
pub(crate) mod console;
mod consts;
pub mod mem;
pub(crate) mod vec;

pub(crate) use archif::ArchIf;
pub(crate) use somehal_macros::println;
