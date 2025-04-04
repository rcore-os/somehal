use core::fmt::Debug;

pub const OFFSET_LINER: usize = 0xffff_8000_0000_0000;
pub const STACK_TOP: usize = 0xffff_f000_0000_0000;

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct AccessFlags: u8 {
        const Read = 1;
        const Write = 1 << 2;
        const Execute = 1 << 3;
        const LowerRead = 1 << 4;
        const LowerWrite = 1 << 5;
        const LowerExecute = 1 << 6;
    }
}

impl Debug for AccessFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}{}{} |L: {}{}{}",
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
            if self.contains(AccessFlags::LowerRead) {
                "R"
            } else {
                "-"
            },
            if self.contains(AccessFlags::LowerWrite) {
                "W"
            } else {
                "-"
            },
            if self.contains(AccessFlags::LowerExecute) {
                "X"
            } else {
                "-"
            },
        )
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheConfig {
    Normal,
    Device,
    /// 需要强制写入主存的场景（如 DMA 缓冲区）
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
