#![no_std]

use log::{LevelFilter, info};
use somehal::{CpuIdx, mem::cpu_idx_to_id, println};

extern crate somehal;

pub mod lang_items;

#[somehal::entry]
fn main(cpu_id: usize, cpu_idx: usize) -> ! {
    println!("Hello, world!");
    println!("cpu_id: {:?}", cpu_id);
    println!("cpu_idx: {:?}", cpu_idx);

    println!("mem cpu_idx: {:?}", somehal::mem::cpu_id());

    if cpu_idx == 0 {
        lang_items::init_heap();

        log::set_logger(&Logger).unwrap();
        log::set_max_level(LevelFilter::Trace);

        info!("log init");

        unsafe {
            somehal::init();
        }

        somehal::mp::cpu_on(cpu_idx_to_id(CpuIdx::new(1)));

        // info!("per id : {:?}", somehal::mem::cpu_id());
        somehal::power::idle();

    } else {
        println!("main cpu_id: {:?}", somehal::mem::cpu_main_id());
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
