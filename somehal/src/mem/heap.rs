use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

pub use linked_list_allocator::LockedHeap;

pub static HEAP: LockedHeap = LockedHeap::empty();

pub unsafe fn alloc(layout: Layout) -> Option<NonNull<u8>> {
    HEAP.lock().allocate_first_fit(layout).ok()
}

#[derive(Clone, Copy)]
pub struct GlobalHeap;

unsafe impl Allocator for GlobalHeap {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { alloc(layout).map(|ptr| NonNull::slice_from_raw_parts(ptr, layout.size())) }
            .ok_or(AllocError {})
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { HEAP.lock().deallocate(ptr, layout) };
    }
}
