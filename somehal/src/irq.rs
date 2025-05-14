use alloc::collections::BTreeMap;

use rdrive::{DeviceId, intc};

use crate::once_static::OnceStatic;

pub use rdrive::intc::{IrqConfig, IrqId};

static IRQ_CPU_MAP: OnceStatic<BTreeMap<DeviceId, intc::BoxCPU>> = OnceStatic::new();

pub(crate) fn init() {
    let mut all = BTreeMap::new();
    for chip in rdrive::dev_list!(Intc) {
        let mut g = chip.spin_try_borrow_by(0.into()).unwrap();
        g.open().unwrap();
        let cpu = g.cpu_interface();
        cpu.setup();
        all.insert(chip.descriptor.device_id, cpu);
    }
    unsafe { IRQ_CPU_MAP.set(all) };
}

pub(crate) fn init_secondary() {
    for chip in rdrive::dev_list!(Intc) {
        if let Some(i) = interface(chip.descriptor.device_id) {
            i.setup();
        }
    }
}

pub fn interface(chip: DeviceId) -> Option<&'static intc::BoxCPU> {
    IRQ_CPU_MAP.as_ref().get(&chip)
}
