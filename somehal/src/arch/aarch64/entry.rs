use crate::{fdt, handle_err, mem::setup_memory_main, println};

pub fn mmu_entry() {
    #[cfg(not(feature = "early-debug"))]
    arch::Arch::init_debugcon();
    println!("MMU ready!");

    let cpu_count = handle_err!(fdt::cpu_count(), "could not get cpu count");

    println!("{:<12}: {}", "CPU count", cpu_count);

    let memories = handle_err!(fdt::find_memory(), "could not get memories");

    unsafe {
        setup_memory_main(memories, cpu_count);


        
    }
}
