use crate::ArchIf;
mod boot;
pub mod debug;
mod paging;

pub struct Arch;

impl ArchIf for Arch {
    fn early_write_str_list(str_list: impl Iterator<Item = &'static str>) {
        debug::write_str_list(str_list);
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

fn rust_main() {}
