use core::error::Error;

use crate::{archif::CpuId, once_static::OnceStatic};

pub type CpuOnFn =
    fn(cpu: CpuId, entry: usize, stack_top: usize) -> Result<(), alloc::boxed::Box<dyn Error>>;

static CPU_ON_FN: OnceStatic<CpuOnFn> = OnceStatic::new();

pub fn init(f: CpuOnFn) {
    unsafe { CPU_ON_FN.init(f) };
}

pub fn cpu_on(
    cpu: CpuId,
    entry: usize,
    stack_top: usize,
) -> Result<(), alloc::boxed::Box<dyn Error>> {
    (CPU_ON_FN)(cpu, entry, stack_top)
}
