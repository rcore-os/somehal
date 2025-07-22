#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheKind {
    Normal,
    Device,
    NoCache,
}

impl CacheKind {
    pub fn mair_idx(&self) -> u64 {
        match self {
            CacheKind::Device => 0,
            CacheKind::Normal => 1,
            CacheKind::NoCache => 2,
        }
    }
}
