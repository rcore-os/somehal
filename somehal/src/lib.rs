#![cfg_attr(not(test), no_std)]
#![feature(naked_functions)]
#![feature(concat_idents)]
#![feature(used_with_arg)]
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
pub mod irq;
pub mod mem;
pub mod mp;
pub(crate) mod once_static;
pub(crate) mod platform;
pub mod power;
pub mod systick;

pub(crate) use archif::ArchIf;

pub use archif::CpuId;
use log::trace;
use mem::page::set_is_relocated;
use mp::CpuOnArg;
pub use percpu;
pub use platform::CpuIdx;
pub use rdrive as driver;
pub use somehal_macros::{entry, module_driver};

pub(crate) fn to_main(arg: &CpuOnArg) -> ! {
    unsafe extern "C" {
        fn __somehal_main(cpu_id: CpuId, cpu_idx: CpuIdx) -> !;
    }
    unsafe {
        set_is_relocated();
        if arg.cpu_idx.is_primary() {
            percpu::init_data(mem::cpu_count());
        }
        mem::setup_arg(arg);
        println!("[SomeHAL] cpu {:?} is ready!", arg.cpu_idx);
        __somehal_main(arg.cpu_id, arg.cpu_idx);
    }
}

pub(crate) fn init_secondary(arg: &CpuOnArg) {
    irq::init_secondary();
    to_main(arg)
}

/// Init hal
/// # Safety
/// This function must be called after the `#[global_allocater]` is initialized, and before device usages.
pub unsafe fn init() {
    platform::init_rdrive();

    driver::register_append(&mem::driver_registers());

    trace!("driver register append");

    driver::probe_pre_kernel().unwrap();

    irq::init();

    systick::init();

    driver::probe().unwrap();
}
