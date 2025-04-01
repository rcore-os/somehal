pub trait ArchIf {
    fn early_write_str_list(str_list: impl Iterator<Item = &'static str>);
    fn is_mmu_enabled() -> bool;
}
