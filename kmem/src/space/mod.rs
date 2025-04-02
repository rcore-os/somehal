use core::fmt::Debug;

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct AccessFlags: u8 {
        const Read = 1;
        const Write = 1 << 2;
        const Execute = 1 << 3;
        const LowerPrivilege = 1 << 4;
    }
}

impl Debug for AccessFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            if self.contains(AccessFlags::Read) {
                "R"
            } else {
                "-"
            },
            if self.contains(AccessFlags::Write) {
                "W"
            } else {
                "-"
            },
            if self.contains(AccessFlags::Execute) {
                "X"
            } else {
                "-"
            },
            if self.contains(AccessFlags::LowerPrivilege) {
                "L"
            } else {
                "-"
            }
        )
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheConfig {
    Normal,
    NoCache,
    WriteThrough,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemConfig {
    pub access: AccessFlags,
    pub cache: CacheConfig,
}

impl Debug for MemConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?} {:?}", self.access, self.cache)
    }
}

#[repr(u8)]
pub enum KSpaceKind {
    Code,
    Stack,
    PerCpu,
    Liner,
}
