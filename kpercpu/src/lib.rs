#![no_std]

mod data;

use core::ptr::NonNull;

pub use data::PerCpuData;
pub use kpercpu_macros::def_percpu;

static mut PERCPU_SIZE: usize = 0;

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

#[macro_export]
macro_rules! impl_percpu {
    ($impl:ty) => {
        #[unsafe(no_mangle)]
        #[inline]
        pub extern "C" fn _percpu_base() -> *mut u8 {
            <$impl as kpercpu::Impl>::percpu_base().as_ptr()
        }
        #[unsafe(no_mangle)]
        #[inline]
        pub extern "C" fn _percpu_set_cpu_local_ptr(ptr: *mut u8) {
            <$impl as kpercpu::Impl>::set_cpu_local_ptr(ptr)
        }
        #[unsafe(no_mangle)]
        #[inline]
        pub extern "C" fn _percpu_get_cpu_local_ptr() -> *mut u8 {
            <$impl as kpercpu::Impl>::get_cpu_local_ptr()
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
    unsafe { PERCPU_SIZE }
}

#[inline]
fn get_cpu_local_ptr() -> *mut u8 {
    unsafe { _percpu_get_cpu_local_ptr() }
}

pub fn init_data(cpu_count: usize) {
    unsafe {
        PERCPU_SIZE = _percpu_load_end as usize - _percpu_load_start as usize;

        let src = core::slice::from_raw_parts(percpu_link_start() as *const u8, percpu_size());

        for i in 0..cpu_count {
            let ptr = (percpu_base() + i * PERCPU_SIZE) as *mut u8;

            let dst = core::slice::from_raw_parts_mut(ptr, percpu_size());

            if i == 0 && src == dst {
                return;
            }
            dst.copy_from_slice(src);
        }
    }
}
pub fn init(cpu_idx: usize) {
    unsafe {
        let ptr = (percpu_base() + cpu_idx * percpu_size()) as *mut u8;
        _percpu_set_cpu_local_ptr(ptr);
    }
}
