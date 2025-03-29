pub trait ArchIf {
    fn early_write_str_list(str_list: impl Iterator<Item = &'static str>);
}
