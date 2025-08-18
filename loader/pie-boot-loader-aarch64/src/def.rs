use core::mem::MaybeUninit;

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

#[repr(C, align(64))]
#[derive(Clone)]
pub struct EarlyBootArgs {
    pub args: [usize; 4],
    pub virt_entry: *mut (),
    pub kimage_addr_lma: *mut (),
    pub kimage_addr_vma: *mut (),
    pub kcode_end: *mut (),
    pub el: usize,
    pub kliner_offset: usize,
    pub page_size: usize,
    pub debug: usize,
}

impl EarlyBootArgs {
    pub const fn new() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }

    pub fn debug(&self) -> bool {
        self.debug > 0
    }
}

impl Default for EarlyBootArgs {
    fn default() -> Self {
        Self::new()
    }
}
