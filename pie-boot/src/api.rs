use core::ptr::NonNull;

pub use crate::vec::ArrayVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryKind {
    Reserved,
    Avilable,
}

impl Default for MemoryKind {
    fn default() -> Self {
        Self::Avilable
    }
}

#[derive(Default, Clone)]
pub struct MemoryRegion {
    pub start: usize,
    pub end: usize,
    pub kind: MemoryKind,
}

#[derive(Default, Clone)]
pub struct BootInfo {
    pub cpu_id: usize,
    pub kcode_offset: usize,
    pub highest_address: usize,
    pub memory_regions: ArrayVec<MemoryRegion, 128>,
    pub fdt: Option<NonNull<u8>>,
}
