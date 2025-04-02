#![no_std]

extern crate somehal;

pub mod lang_items;

#[unsafe(no_mangle)]
pub extern "C" fn rust_main(_cpu_id: usize, _dtb: usize) -> ! {
    unimplemented!()
}
