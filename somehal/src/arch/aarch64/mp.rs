use core::error::Error;

use kmem_region::PhysAddr;

use crate::{archif::CpuId, once_static::OnceStatic};

use super::cache;

pub type CpuOnFn =
    fn(cpu: CpuId, entry: usize, stack_top: PhysAddr) -> Result<(), alloc::boxed::Box<dyn Error>>;

static CPU_ON_FN: OnceStatic<CpuOnFn> = OnceStatic::new();

pub fn init(f: CpuOnFn) {
    unsafe { CPU_ON_FN.set(f) };
}

pub fn cpu_on(
    cpu: CpuId,
    entry: usize,
    stack_top: PhysAddr,
) -> Result<(), alloc::boxed::Box<dyn Error>> {
    unsafe { cache::dcache_all(cache::DcacheOp::CleanAndInvalidate) };
    (CPU_ON_FN)(cpu, entry, stack_top)
}
