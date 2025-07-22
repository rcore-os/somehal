mod mmu;

pub use crate::common::mem::phys_to_virt;

// After GlobalAlloc is implemented, this will be used as the global allocator.
pub fn init() {
    mmu::init_mmu();
}
