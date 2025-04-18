use crate::archif::ArchIf;

mod boot;
mod mmu;
mod uart16550;

use mmu::new_pte_with_config;
use page_table_generic::TableGeneric;

pub struct Arch;

impl ArchIf for Arch {
    #[inline(always)]
    #[allow(deprecated)]
    fn early_debug_put(byte: u8) {
        uart16550::write_bytes(&[byte]);
    }

    fn wait_for_event() {
        loop {}
    }

    type PageTable = mmu::Table;

    fn new_pte_with_config(
        config: kmem::region::MemConfig,
    ) -> <Self::PageTable as TableGeneric>::PTE {
        new_pte_with_config(config)
    }
}
