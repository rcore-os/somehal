use crate::archif::ArchIf;

mod boot;
mod mmu;

pub use boot::*;
use mmu::new_pte_with_config;
use page_table_generic::TableGeneric;

pub struct Arch;

impl ArchIf for Arch {
    #[inline(always)]
    #[allow(deprecated)]
    fn early_debug_put(byte: u8) {
        
    }

    fn wait_for_event() {
       loop{}
    }

    type PageTable = mmu::Table;

    fn new_pte_with_config(
        config: kmem::region::MemConfig,
    ) -> <Self::PageTable as TableGeneric>::PTE {
        new_pte_with_config(config)
    }
}
