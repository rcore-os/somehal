use somehal_macros::dbgln;

use crate::{ArchIf, arch::Arch, println};

pub fn mmu_entry() -> ! {
    dbgln!("MMU ready!");
    // println!("MMU ready!");

    loop {
        Arch::wait_for_event();
    }
}
