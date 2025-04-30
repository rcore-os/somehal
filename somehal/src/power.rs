use rdrive::DriverKind;
use rdrive::power::*;

use crate::ArchIf;
use crate::arch::Arch;

pub fn init() {
    rdrive::probe_with_kind(DriverKind::Power).unwrap();

    use_power(|p| p.open()).unwrap();
}

fn use_power<T, F: FnOnce(&mut Hardware) -> T>(f: F) -> T {
    let power_list = rdrive::read(|m| m.power.all());

    let power = power_list
        .first()
        .expect("No power driver found")
        .1
        .upgrade()
        .expect("Power driver droped");
    let mut g = power.spin_try_borrow_by(0.into());
    (f)(&mut g)
}

pub fn terminate() -> ! {
    use_power(|p| p.shutdown());
    loop {
        Arch::wait_for_event();
    }
}
