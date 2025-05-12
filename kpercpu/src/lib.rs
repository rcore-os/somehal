#![no_std]

mod data;

use core::ptr::NonNull;

pub use data::PerCpuData;

unsafe extern "C" {
    fn _percpu_load_start();
    fn _percpu_load_end();

    fn _percpu_base() -> *mut u8;
    fn _percpu_set_cpu_local_ptr(ptr: *mut u8);
    fn _percpu_get_cpu_local_ptr() -> *mut u8;
}

pub trait Impl {
    fn percpu_base() -> NonNull<u8>;
    fn set_cpu_local_ptr(ptr: *mut u8);
    fn get_cpu_local_ptr() -> *mut u8;
}

macro_rules! impl_percpu {
    ($impl:ty) => {
        #[unsafe(no_mangle)]
        #[inline]
        pub extern "C" fn _percpu_base() -> *mut u8 {
            <impl as Impl>::percpu_base().as_ptr()
        }
        #[unsafe(no_mangle)]
        #[inline]
        pub extern "C" fn _percpu_set_cpu_local_ptr(ptr: *mut u8) {
            <impl as Impl>::set_cpu_local_ptr(ptr)
        }
        #[unsafe(no_mangle)]
        #[inline]
        pub extern "C" fn _percpu_get_cpu_local_ptr() -> *mut u8 {
            <impl as Impl>::get_cpu_local_ptr()
        }
    };
}

#[inline]
fn percpu_base() -> usize {
    unsafe { _percpu_base() as usize }
}
#[inline]
fn percpu_link_start() -> usize {
    _percpu_load_start as usize
}
#[inline]
fn percpu_size() -> usize {
    _percpu_load_end as usize - _percpu_load_start as usize
}

#[inline]
fn get_cpu_local_ptr() -> *mut u8 {
    unsafe { _percpu_get_cpu_local_ptr() }
}
