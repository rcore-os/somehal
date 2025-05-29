use rdrive::power::*;

use crate::ArchIf;
use crate::arch::Arch;

fn use_power<T, F: FnOnce(&mut Boxed) -> T>(f: F) -> T {
    let weak = match rdrive::get_dev!(Power) {
        Some(v) => v,
        None => {
            crate::println!("No power driver found!");
            loop {
                Arch::wait_for_event();
            }
        }
    };

    let mut dev = weak.spin_try_borrow_by(0.into()).unwrap();
    (f)(&mut dev)
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
