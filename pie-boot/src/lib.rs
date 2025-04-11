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

#[cfg(feature = "early-debug")]
mod early_debug;
mod once_static;

pub use arch::*;

fn bss_mut() -> &'static mut [u8] {
    unsafe extern "C" {
        fn __start_bss();
        fn __stop_bss();
    }
    unsafe {
        let start = __start_bss as *mut u8;
        let end = __stop_bss as *mut u8;

        core::slice::from_raw_parts_mut(start, end as usize - start as usize)
    }
}

unsafe fn clean_bss() {
    bss_mut().fill(0);
}
