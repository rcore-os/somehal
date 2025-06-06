use core::cell::UnsafeCell;

use any_uart::Sender;

#[cfg(fdt)]
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
