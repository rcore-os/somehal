use rdrive::power::*;

use crate::ArchIf;
use crate::arch::Arch;

fn use_power<T, F: FnOnce(&mut Hardware) -> T>(f: F) -> T {
    let mut g = rdrive::get_dev!(Power)
        .expect("No power driver found")
        .spin_try_borrow_by(0.into())
        .unwrap();
    (f)(&mut g)
}

pub fn terminate() -> ! {
    use_power(|p| p.shutdown());
    loop {
        Arch::wait_for_event();
    }
}

pub fn idle() -> ! {
    loop {
        Arch::wait_for_event();
    }
}
