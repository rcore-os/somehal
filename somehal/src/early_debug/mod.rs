use core::fmt::Write;
use spin::Mutex;
use crate::lazy_static::LazyStatic;

type RxFun = fn() -> Result<u8, RError>;

#[unsafe(link_section = ".data")]
static RX_FUN: LazyStatic<RxFun> = LazyStatic::with_default(empty_rx);

#[unsafe(link_section = ".data")]
static RX_MUTEX: Mutex<()> = Mutex::new(());

type TxFun = fn(u8) -> Result<(), TError>;

#[unsafe(link_section = ".data")]
static TX_FUN: LazyStatic<TxFun> = LazyStatic::with_default(empty_tx);
#[unsafe(link_section = ".data")]
static TX_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy)]
pub enum RError {
    NoData,
    Timeout,
    ParityError,
    FrameError,
    Overrun,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub enum TError {
    ReTry,
    Other,
}

fn empty_rx() -> Result<u8, RError> {
    Err(RError::NoData)
}

fn empty_tx(_: u8) -> Result<(), TError> {
    Ok(())
}

pub(crate) fn set_rx_fun(rx: RxFun) {
    RX_FUN.init(rx);
}

pub(crate) fn set_tx_fun(tx: TxFun) {
    TX_FUN.init(tx);
}

fn _read_byte() -> Result<u8, RError> {
    RX_FUN()
}

pub fn read_byte() -> Result<u8, RError> {
    let _lock = RX_MUTEX.lock();
    loop {
        match _read_byte() {
            Ok(b) => return Ok(b),
            Err(RError::NoData) => continue,
            Err(e) => return Err(e),
        }
    }
}

pub fn read_bytes(buffer: &mut [u8]) -> Result<usize, RError> {
    let _lock = RX_MUTEX.lock();
    let mut count = 0;
    
    for slot in buffer.iter_mut() {
        match _read_byte() {
            Ok(b) => {
                *slot = b;
                count += 1;
            }
            Err(RError::NoData) => break,
            Err(e) => return Err(e),
        }
    }
    
    Ok(count)
}

pub fn read_line(buffer: &mut [u8]) -> Result<usize, RError> {
    let _lock = RX_MUTEX.lock();
    let mut count = 0;
    let mut prev_was_cr = false;
    
    for slot in buffer.iter_mut() {
        loop {
            match _read_byte() {
                Ok(b) => {
                    if b == b'\n' {
                        return Ok(count);
                    }
                    
                    if b == b'\r' {
                        prev_was_cr = true;
                        continue;
                    }

                    if prev_was_cr {
                        *slot = b;
                        count += 1;
                        return Ok(count);
                    }
                    
                    *slot = b;
                    count += 1;
                    break;
                }
                Err(RError::NoData) => continue,
                Err(e) => return Err(e),
            }
        }
    }
    
    Ok(count)
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

pub fn write_bytes(bytes: &[u8]) -> Result<(), TError> {
    let _lock = TX_MUTEX.lock();
    for &b in bytes {
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
