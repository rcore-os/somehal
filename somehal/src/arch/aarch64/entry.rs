use super::debug;
use crate::{fdt, handle_err, println};

pub fn mmu_entry() {
    debug::init();
    println!("MMU ready!");

    let cpu_count = handle_err!(fdt::cpu_count(), "could not get cpu count");

    println!("{:<12}: {}", "CPU count", cpu_count);

    // let memories = handle_err!(fdt::find_memory(), "could not get memories");
}
