use core::cell::UnsafeCell;

use any_uart::Sender;
use kdef_pgtable::PAGE_SIZE;
use num_align::NumAlign;
use pie_boot_if::{DebugConsole, String, Vec};

use crate::RETURN;

pub mod fdt;

pub static mut REG_BASE: usize = 0;

pub fn reg_base() -> usize {
    unsafe { REG_BASE }
}

fn setup_debugcon<'a>(base: usize, compatibles: impl core::iter::Iterator<Item = &'a str>) {
    unsafe {
        REG_BASE = base.align_down(PAGE_SIZE);
        let mut ls = Vec::new();
        for c in compatibles {
            let mut s = String::new();
            let _ = s.push_str(c);
            let _ = ls.push(s);
        }
        RETURN.as_mut().debug_console = Some(DebugConsole {
            base: base as _,
            compatibles: ls,
        });
    }
}

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
