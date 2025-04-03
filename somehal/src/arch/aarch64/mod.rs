use somehal_macros::println;

use crate::ArchIf;
mod boot;
mod context;
pub mod debug;
mod paging;
mod trap;

pub struct Arch;

impl ArchIf for Arch {
    fn early_debug_put(b: u8) {
        debug::write_byte(b);
    }

    fn is_mmu_enabled() -> bool {
        paging::is_mmu_enabled()
    }

    type PageTable = paging::Table;

    fn new_pte_with_config(
        config: kmem::space::MemConfig,
    ) -> <Self::PageTable as page_table_generic::TableGeneric>::PTE {
        paging::new_pte_with_config(config)
    }
}

fn rust_main() {
    println!("reloacte ok");
}
