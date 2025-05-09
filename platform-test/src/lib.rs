#![no_std]

use core::hint::spin_loop;

use log::{LevelFilter, info};
use somehal::println;

extern crate somehal;

pub mod lang_items;

#[somehal::entry]
fn main(cpu_id: usize, cpu_idx: usize) -> ! {
    println!("Hello, world!");
    println!("cpu_id: {:?}", cpu_id);
    println!("cpu_idx: {:?}", cpu_idx);

    if cpu_idx == 0 {
        lang_items::init_heap();

        log::set_logger(&Logger).unwrap();
        log::set_max_level(LevelFilter::Trace);

        info!("log init");

        unsafe {
            somehal::init();
        }

        somehal::mp::cpu_on(1.into());

        // info!("per id : {:?}", somehal::mem::cpu_id());

        somehal::power::idle();
    } else {
        println!("all test passed!");
        somehal::power::terminate();
        // info!("per id: {:?}", somehal::mem::cpu_id());
    }
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
