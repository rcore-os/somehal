use crate::{ArchIf, arch::Arch, archif::CpuId};

pub fn cpu_on(target: CpuId) {
    Arch::start_secondary_cpu(target, 0, 0).unwrap();
}

