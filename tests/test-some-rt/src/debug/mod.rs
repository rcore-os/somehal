use core::{cell::UnsafeCell, fmt::Write};

use any_uart::Sender;
use log::Log;

pub mod fdt;

static UART: UartWapper = UartWapper(UnsafeCell::new(None));

struct UartWapper(UnsafeCell<Option<Sender>>);

unsafe impl Send for UartWapper {}
unsafe impl Sync for UartWapper {}

impl UartWapper {
    fn set(&self, uart: Sender) {
        unsafe {
            *self.0.get() = Some(uart);
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn get(&self) -> Option<&mut Sender> {
        unsafe { &mut *self.0.get() }.as_mut()
    }
}

fn set_uart(uart: any_uart::Uart) -> Option<()> {
    let tx = uart.tx?;
    UART.set(tx);
    Some(())
}

pub fn write_byte(b: u8) {
    if let Some(tx) = UART.get() {
        let _ = any_uart::block!(tx.write(b));
    }
}
struct TX;

impl Write for TX {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            write_byte(b);
        }
        Ok(())
    }
}
struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let _ = TX {}.write_fmt(format_args!("[{}] {}", record.level(), record.args()));
    }

    fn flush(&self) {}
}

pub fn init_log() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}
