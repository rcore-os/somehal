#![cfg_attr(not(all(test, target_os = "none")), no_std)]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]
#![feature(cfg_match)]
#![feature(fn_align)]
#![feature(pointer_is_aligned_to)]

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

mod archif;
mod config;
mod mem;
#[allow(unused)]
mod paging;

use core::ptr::NonNull;

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

use kmem_region::PhysAddr;
use mem::{boot_info, boot_info_addr};
#[cfg(early_debug)]
pub(crate) use somehal_macros::dbgln;

#[cfg(not(early_debug))]
#[macro_export]
macro_rules! dbgln {
    ($($arg:tt)*) => {};
}

#[derive(Default, Clone)]
pub struct BootInfo {
    pub cpu_id: usize,
    pub kcode_offset: usize,
    pub fdt: Option<NonNull<u8>>,
    pub main_memory_free_start: PhysAddr,
}

impl BootInfo {
    pub const fn new() -> Self {
        Self {
            cpu_id: 0,
            kcode_offset: 0,
            fdt: None,
            main_memory_free_start: PhysAddr::new(0),
        }
    }
}

pub(crate) fn relocate() {
    unsafe extern "Rust" {
        fn __vma_relocate_entry(boot_info: BootInfo);
    }

    unsafe {
        __vma_relocate_entry(boot_info());
    }
}
