use log::Log;
use somehal::println;

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!("[{}] {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

pub fn init_log() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}
