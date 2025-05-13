pub use crate::mem::percpu::cpu_list;
use kmem_region::{PhysAddr, VirtAddr};

use crate::{
    ArchIf,
    arch::Arch,
    archif::CpuId,
    mem::{cpu_id_to_idx, stack_top_phys},
    platform::CpuIdx,
};

pub(crate) struct CpuOnArg {
    pub cpu_id: CpuId,
    pub cpu_idx: CpuIdx,
    pub stack_top_virt: VirtAddr,
    pub boot_table: PhysAddr,
}

pub fn cpu_on(target: CpuId) {
    let stack_top = stack_top_phys(cpu_id_to_idx(target));
    let stack = stack_top - size_of::<CpuOnArg>();

    Arch::start_secondary_cpu(target, stack).unwrap();
}
