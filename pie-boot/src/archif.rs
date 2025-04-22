#![allow(unused)]

use crate::paging::TableGeneric;
use kmem_region::region::MemConfig;

pub trait ArchIf {
    fn early_debug_put(byte: u8);
    fn wait_for_event();

    type PageTable: TableGeneric;
    fn new_pte_with_config(config: MemConfig) -> <Self::PageTable as TableGeneric>::PTE;
}
