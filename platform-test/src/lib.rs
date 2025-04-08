#![no_std]

use somehal::println;

extern crate somehal;

pub mod lang_items;

#[somehal::entry]
fn main(_cpu_id: usize, _dtb: usize) -> ! {
    println!("Hello, world!");

    unimplemented!()
}
