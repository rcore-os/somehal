use page_table_generic::Access;

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
}

impl Access for LineAllocator {
    #[inline(always)]
    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
        LineAllocator::alloc(self, layout)
    }

    #[inline(always)]
    unsafe fn dealloc(&mut self, _ptr: PhysAddr, _layout: core::alloc::Layout) {}

    #[inline(always)]
    fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
        phys.raw() as _
    }
}
