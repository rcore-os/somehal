use core::{alloc::Layout, ptr::NonNull};

pub use linked_list_allocator::LockedHeap;

pub static HEAP: LockedHeap = LockedHeap::empty();

pub unsafe fn alloc(layout: Layout) -> Option<NonNull<u8>> {
    HEAP.lock().allocate_first_fit(layout).ok()
}
