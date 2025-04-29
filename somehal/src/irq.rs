use alloc::collections::BTreeMap;

pub use rdrive::intc_all;
use rdrive::{DeviceId, DriverKind, intc};

use crate::once_static::OnceStatic;

static IRQ_CPU_MAP: OnceStatic<BTreeMap<DeviceId, intc::HardwareCPU>> = OnceStatic::new();

pub(crate) fn init() {
    rdrive::probe_with_kind(DriverKind::Intc).unwrap();
    let mut all = BTreeMap::new();
    for (id, chip) in rdrive::intc_all() {
        let ptr = chip.upgrade().unwrap();
        let mut chip = ptr.spin_try_borrow_by(0.into());
        chip.open().unwrap();
        let cpu = chip.cpu_interface();
        all.insert(id, cpu);
    }
    unsafe { IRQ_CPU_MAP.init(all) };
}

pub fn interface(chip: DeviceId) -> Option<&'static intc::HardwareCPU> {
    IRQ_CPU_MAP.as_ref().get(&chip)
}
