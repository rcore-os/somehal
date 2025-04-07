use crate::{ArchIf, arch::Arch, println};

pub fn mmu_entry() -> ! {
    println!("MMU ready!");

    loop {
        Arch::wait_for_event();
    }
}
