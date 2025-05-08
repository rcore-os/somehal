use rdrive::power::*;

use crate::ArchIf;
use crate::arch::Arch;

fn use_power<T, F: FnOnce(&mut Hardware) -> T>(f: F) -> T {
    let power_list = rdrive::dev_list!(Power);

    let power = power_list.first().expect("No power driver found");

    let mut g = power.spin_try_borrow_by(0.into()).unwrap();
    (f)(&mut g)
}

pub fn terminate() -> ! {
    use_power(|p| p.shutdown());
    loop {
        Arch::wait_for_event();
    }
}
