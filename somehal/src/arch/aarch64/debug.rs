#[link_boot::link_boot]
mod _m {

    use core::ptr::NonNull;

    use any_uart::Sender;
    use spin::Mutex;

    static TX: Mutex<Option<Sender>> = Mutex::new(None);

    pub(crate) fn set_uart(uart: any_uart::Uart) -> Option<()> {
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
}
