use core::ptr::NonNull;

use any_uart::Sender;
use spin::Mutex;

static TX: Mutex<Option<Sender>> = Mutex::new(None);

pub fn init_by_dtb(dtb: *mut u8) -> Option<()> {
    let uart = any_uart::init(NonNull::new(dtb)?, |p| p as _)?;
    TX.lock().replace(uart.tx?);
    Some(())
}

pub fn write_str_list(str_list: impl Iterator<Item = &'static str>) {
    let mut g = TX.lock();
    if let Some(tx) = g.as_mut() {
        for s in str_list {
            for &b in s.as_bytes() {
                let _ = any_uart::block!(tx.write(b));
            }
        }
    }
}
