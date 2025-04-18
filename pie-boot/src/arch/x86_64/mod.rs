use core::arch::asm;

use crate::archif::ArchIf;

mod boot;
mod mmu;

use mmu::new_pte_with_config;
use page_table_generic::TableGeneric;

pub struct Arch;

impl ArchIf for Arch {
    fn early_debug_put(byte: u8) {
        todo!()
    }

    fn wait_for_event() {
        unsafe { asm!("hlt") }
    }

    type PageTable = mmu::Table;

    fn new_pte_with_config(
        config: kmem::region::MemConfig,
    ) -> <Self::PageTable as TableGeneric>::PTE {
        new_pte_with_config(config)
    }
}
