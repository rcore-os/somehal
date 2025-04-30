use crate::{ArchIf, arch::Arch};

pub fn current_ticks() -> u64 {
    Arch::current_ticks()
}

pub fn ticks_to_nanos(ticks: u64) -> u128 {
    Arch::ticks_to_nanos(ticks)
}

pub fn nanos_to_ticks(nanos: u128) -> u64 {
    Arch::nanos_to_ticks(nanos)
}
