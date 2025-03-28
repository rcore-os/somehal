#![no_std]

extern crate somehal;

pub mod lang_items;

#[unsafe(no_mangle)]
pub extern "C" fn rust_main(cpu_id: usize, dtb: usize) -> ! {
    unimplemented!()
}
