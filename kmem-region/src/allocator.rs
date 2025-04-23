use crate::{IntAlign, PhysAddr};

pub struct LineAllocator {
    pub start: PhysAddr,
    iter: PhysAddr,
    pub end: PhysAddr,
}

impl LineAllocator {
    #[inline(always)]
    pub fn new(start: PhysAddr, size: usize) -> Self {
        Self {
            start,
            iter: start,
            end: start + size,
        }
    }

    #[inline(always)]
    pub fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
        let start = self.iter.align_up(layout.align());
        if start + layout.size() > self.end {
            return None;
        }
        self.iter = start + layout.size();

        Some(start)
    }

    #[inline(always)]
    pub fn highest_address(&self) -> PhysAddr {
        self.iter
    }

    #[inline(always)]
    pub fn used(&self) -> usize {
        self.iter - self.start
    }
}
