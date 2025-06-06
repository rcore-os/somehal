#![no_std]

extern crate some_rt;

pub mod lang_items;

// #[some_rt::entry]
fn main(cpu_id: usize, cpu_idx: usize) -> ! {
    loop {}
}
