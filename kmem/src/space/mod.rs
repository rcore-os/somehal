bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct AccessFlags: u8 {
        const Read = 1;
        const Write = 1 << 2;
        const LowerPrivilege = 1 << 3;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheSetting {
    Normal,
    NonCache,
    WriteBack,
    WriteThrough,
}
