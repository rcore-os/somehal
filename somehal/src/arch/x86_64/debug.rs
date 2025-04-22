use core::cell::UnsafeCell;

use crate::{mem::MemRegion, once_static::OnceStatic};

use any_uart::Sender;
use kmem_region::region::OFFSET_LINER;

pub(super) static MEM_REGION_DEBUG_CON: OnceStatic<MemRegion> = OnceStatic::new();
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
    fn get(&self) -> &mut Sender {
        unsafe { &mut *self.0.get().as_mut().unwrap().as_mut().unwrap() }
    }
}

pub(crate) fn init() {
    unsafe {
    }
}

fn set_uart(uart: any_uart::Uart) -> Option<()> {
    let tx = uart.tx?;
    UART.set(tx);
    Some(())
}

pub fn write_byte(b: u8) {
    let tx = UART.get();
    let _ = any_uart::block!(tx.write(b));
}
