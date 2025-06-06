#![allow(unused)]

use crate::paging::TableGeneric;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheConfig {
    Normal,
    NoCache,
}

pub trait ArchIf {
    fn early_debug_put(byte: u8);
    fn wait_for_event();

    type PageTable: TableGeneric;
    fn new_pte_with_config(config: CacheConfig) -> <Self::PageTable as TableGeneric>::PTE;
}
