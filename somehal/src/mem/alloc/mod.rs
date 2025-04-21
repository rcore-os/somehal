pub use linked_list_allocator::LockedHeap;

pub static HEAP: LockedHeap = LockedHeap::empty();

pub struct PhysAllocator{

}
