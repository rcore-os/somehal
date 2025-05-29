use rdrive::systick::*;

use crate::{ArchIf, arch::Arch, once_static::OnceStatic};

static TIMER: OnceStatic<local::Boxed> = OnceStatic::new();

pub fn current_ticks() -> u64 {
    Arch::current_ticks()
}

pub fn ticks_to_nanos(ticks: u64) -> u128 {
    Arch::ticks_to_nanos(ticks)
}

pub fn nanos_to_ticks(nanos: u128) -> u64 {
    Arch::nanos_to_ticks(nanos)
}

pub fn get() -> &'static local::Boxed {
    &TIMER
}

pub fn init() -> Option<()> {
    let timer = rdrive::get_dev!(Systick)?;

    let mut g = timer.spin_try_borrow_by(0.into()).ok()?;

    let cpu = g.cpu_local();

    unsafe { TIMER.set(cpu) };

    Some(())
}

pub fn set_enable(b: bool) {
    Arch::systick_set_enable(b);
}
