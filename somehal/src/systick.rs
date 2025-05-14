use rdrive::timer::HardwareCPU;

use crate::{ArchIf, arch::Arch, once_static::OnceStatic};

static TIMER: OnceStatic<HardwareCPU> = OnceStatic::new();

pub fn current_ticks() -> u64 {
    Arch::current_ticks()
}

pub fn ticks_to_nanos(ticks: u64) -> u128 {
    Arch::ticks_to_nanos(ticks)
}

pub fn nanos_to_ticks(nanos: u128) -> u64 {
    Arch::nanos_to_ticks(nanos)
}

pub fn get() -> &'static HardwareCPU {
    &TIMER
}

pub fn init() -> Option<()> {
    let timer = rdrive::get_dev!(Timer)?;

    let mut g = timer.spin_try_borrow_by(0.into()).ok()?;

    let cpu = g.get_current_cpu();

    unsafe { TIMER.set(cpu) };

    Some(())
}

pub fn set_enable(b: bool) {
    Arch::systick_set_enable(b);
}
