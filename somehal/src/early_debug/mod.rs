use core::fmt::Write;

use spin::Mutex;

use crate::lazy_static::LazyStatic;

type TxFun = fn(u8) -> Result<(), TError>;

#[unsafe(link_section = ".data")]
static TX_FUN: LazyStatic<TxFun> = LazyStatic::with_default(empty_tx);
#[unsafe(link_section = ".data")]
static TX_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy)]
pub enum TError {
    ReTry,
    Other,
}

fn empty_tx(_: u8) -> Result<(), TError> {
    Ok(())
}

pub(crate) fn set_tx_fun(tx: TxFun) {
    TX_FUN.init(tx);
}

fn _write_byte(b: u8) -> Result<(), TError> {
    TX_FUN(b)
}

struct TX;

impl core::fmt::Write for TX {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            loop {
                match _write_byte(b) {
                    Ok(_) => break,
                    Err(TError::ReTry) => continue,
                    Err(TError::Other) => break,
                }
            }
        }
        Ok(())
    }
}

pub fn write_str(s: &str) {
    let _lock = TX_MUTEX.lock();
    let mut tx = TX;
    let _ = tx.write_str(s);
}

pub fn write_fmt(args: core::fmt::Arguments) {
    let _lock = TX_MUTEX.lock();
    let mut tx = TX;
    let _ = tx.write_fmt(args);
}

pub fn write_byte(b: u8) -> Result<(), TError> {
    let _lock = TX_MUTEX.lock();
    _write_byte(b)
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::early_debug::write_fmt(format_args!("\r\n"));
    };
    ($($arg:tt)*) => {
        $crate::early_debug::write_fmt(format_args!($($arg)*));
        $crate::early_debug::write_fmt(format_args!("\r\n"));
    };
}
