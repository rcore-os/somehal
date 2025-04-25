#![no_std]

use somehal::{mem::cpu_id, println};
use spin::Mutex;

extern crate somehal;

pub mod lang_items;

#[somehal::entry]
fn main(_cpu_id: usize, _dtb: usize) -> ! {
    println!("Hello, world!");
    println!("cpu_id: {:?}", cpu_id());

    unimplemented!()
}
