use kmem::space::MemConfig;
pub use page_table_generic::*;

pub trait ArchIf {
    fn early_write_str_list(str_list: impl Iterator<Item = &'static str>);
    fn is_mmu_enabled() -> bool;
    type PageTable: TableGeneric;
    fn new_pte_with_config(config: MemConfig) -> <Self::PageTable as TableGeneric>::PTE;
}
