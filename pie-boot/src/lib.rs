#![no_std]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]
#![feature(cfg_match)]

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
#[cfg(early_debug)]
pub(crate) mod debug;

mod archif;
mod config;
mod mem;

pub use arch::*;

unsafe fn clean_bss() {
    unsafe extern "C" {
        fn __start_bss();
        fn __stop_bss();
    }
    unsafe {
        let start = __start_bss as *mut u8;
        let end = __stop_bss as *mut u8;
        let len = end as usize - start as usize;
        for i in 0..len {
            start.add(i).write(0);
        }
    }
}

#[cfg(early_debug)]
pub(crate) use somehal_macros::dbgln;

#[cfg(not(early_debug))]
#[macro_export]
macro_rules! dbgln {
    ($($arg:tt)*) => {};
}
