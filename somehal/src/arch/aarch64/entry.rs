use crate::println;

pub fn mmu_entry() {
    #[cfg(not(feature = "early-debug"))]
    arch::Arch::init_debugcon();
    println!("Hello, world!");
}
