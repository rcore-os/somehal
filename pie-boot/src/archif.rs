#![allow(unused)]

use kmem::region::MemConfig;
use page_table_generic::TableGeneric;

pub trait ArchIf {
    fn early_debug_put(byte: u8);
    fn wait_for_event();

    type PageTable: TableGeneric;
    fn new_pte_with_config(config: MemConfig) -> <Self::PageTable as TableGeneric>::PTE;
}
