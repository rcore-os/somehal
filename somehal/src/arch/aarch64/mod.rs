use crate::ArchIf;

mod boot;
pub mod debug;

pub struct Arch;

impl ArchIf for Arch {
    fn early_write_str_list(str_list: impl Iterator<Item = &'static str>) {
        debug::write_str_list(str_list);
    }
}
