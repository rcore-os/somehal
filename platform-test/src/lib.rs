#![no_std]

use log::{LevelFilter, info};
use somehal::{mem::cpu_id, println};

extern crate somehal;

pub mod lang_items;

#[somehal::entry]
fn main(_cpu_id: usize, _dtb: usize) -> ! {
    println!("Hello, world!");
    println!("cpu_id: {:?}", cpu_id());

    lang_items::init_heap();

    log::set_logger(&Logger).unwrap();
    log::set_max_level(LevelFilter::Trace);

    info!("log init");

    unsafe {
        somehal::init();
    }

    unimplemented!()
}

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
