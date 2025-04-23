#![allow(unused)]

pub mod vec {
    pub type Vec<T> = alloc::vec::Vec<T, crate::mem::heap::GlobalHeap>;
}
pub mod boexd {
    pub type Box<T> = alloc::boxed::Box<T, crate::mem::heap::GlobalHeap>;
}

#[macro_export]
macro_rules!  vec{
     () => (
        $crate::_alloc::vec::Vec::new_in($crate::mem::heap::GlobalHeap)
    );
    ($elem:expr; $n:expr) => (
        $crate::_alloc::vec::from_elem_in($elem, $n, $crate::mem::heap::GlobalHeap)
    );
    ($($x:expr),+ $(,)?) => (
        <[_]>::into_vec_in(
            // Using the intrinsic produces a dramatic improvement in stack usage for
            // unoptimized programs using this code path to construct large Vecs.
            $crate::_alloc::boxed::box_new_in([$($x),+], $crate::mem::heap::GlobalHeap), $crate::mem::heap::GlobalHeap
        )
    );
}
