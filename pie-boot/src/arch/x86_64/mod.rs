use core::arch::asm;

mod boot;
mod mmu;

use kmem_region::region::MemConfig;

use crate::archif::ArchIf;
use crate::paging::TableGeneric;
use mmu::new_pte_with_config;

pub struct Arch;

impl ArchIf for Arch {
    fn early_debug_put(_byte: u8) {
        todo!()
    }

    fn wait_for_event() {
        unsafe { asm!("hlt") }
    }

    type PageTable = mmu::Table;

    fn new_pte_with_config(config: MemConfig) -> <Self::PageTable as TableGeneric>::PTE {
        new_pte_with_config(config)
    }
}
