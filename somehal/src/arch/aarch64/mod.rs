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
}

fn rust_main() {}
