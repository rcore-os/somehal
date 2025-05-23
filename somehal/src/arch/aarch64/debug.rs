use core::cell::UnsafeCell;

use crate::platform;

use any_uart::{Receiver, Sender};
use kmem_region::region::OFFSET_LINER;

static TX: UartWapper<Sender> = UartWapper(UnsafeCell::new(None));
static RX: UartWapper<Receiver> = UartWapper(UnsafeCell::new(None));

struct UartWapper<T>(UnsafeCell<Option<T>>);

unsafe impl<T> Send for UartWapper<T> {}
unsafe impl<T> Sync for UartWapper<T> {}

impl<T> UartWapper<T> {
    fn set(&self, uart: T) {
        unsafe {
            *self.0.get() = Some(uart);
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get().as_mut().unwrap().as_mut().unwrap() }
    }
}

pub(crate) fn init() {
    let uart = platform::init_debugcon().unwrap();
    set_uart(uart);
}

fn set_uart(uart: any_uart::Uart) -> Option<()> {
    let tx = uart.tx?;
    let rx = uart.rx?;
    TX.set(tx);
    RX.set(rx);
    Some(())
}

pub(crate) fn reloacte() {
    TX.get().mmio_base_add(OFFSET_LINER);
    RX.get().mmio_base_add(OFFSET_LINER);
}

pub fn write_byte(b: u8) {
    let tx = TX.get();
    let _ = any_uart::block!(tx.write(b));
}

pub fn get_byte() -> Option<u8> {
    let rx = RX.get();
    any_uart::block!(rx.read()).ok()
}
