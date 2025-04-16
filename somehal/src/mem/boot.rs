use kmem::{
    IntAlign,
    paging::{Access, PhysAddr},
};

use super::PhysMemory;

static mut KCODE_VA_OFFSET: usize = 0;

pub struct LineAllocator {
    pub start: PhysAddr,
    iter: PhysAddr,
    pub end: PhysAddr,
}

impl LineAllocator {
    pub fn new(start: PhysAddr, size: usize) -> Self {
        Self {
            start,
            iter: start,
            end: start + size,
        }
    }

    pub fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
        let start = self.iter.align_up(layout.align());
        if start + layout.size() > self.end {
            return None;
        }

        self.iter += layout.size().align_up(layout.align());

        Some(start)
    }

    pub fn used(&self) -> PhysMemory {
        PhysMemory {
            addr: self.start,
            size: self.iter - self.start,
        }
    }
}

impl Access for LineAllocator {
    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
        LineAllocator::alloc(self, layout)
    }

    unsafe fn dealloc(&mut self, _ptr: PhysAddr, _layout: core::alloc::Layout) {}

    fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
        phys.raw() as _
    }
}

pub unsafe fn set_kcode_va_offset(offset: usize) {
    unsafe { KCODE_VA_OFFSET = offset };
}

pub fn kcode_offset() -> usize {
    unsafe { KCODE_VA_OFFSET }
}
