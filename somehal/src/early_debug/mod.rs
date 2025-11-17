use crate::lazy_static::LazyStatic;
use core::fmt::Write;
use spin::Mutex;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RError {
    NoData,
    Timeout,
    ParityError,
    FrameError,
    Overrun,
    BufferFull,
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

#[inline]
fn _read_byte() -> Result<u8, RError> {
    RX_FUN()
}

pub fn try_read_byte() -> Result<u8, RError> {
    let _lock = RX_MUTEX.lock();
    _read_byte()
}

pub fn read_byte() -> Result<u8, RError> {
    let _lock = RX_MUTEX.lock();
    loop {
        match _read_byte() {
            Ok(b) => return Ok(b),
            Err(RError::NoData) => {
                core::hint::spin_loop();
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

/// 带超时的阻塞读取（max_spins 为自旋次数）
pub fn read_byte_timeout(max_spins: usize) -> Result<u8, RError> {
    let _lock = RX_MUTEX.lock();
    for _ in 0..max_spins {
        match _read_byte() {
            Ok(b) => return Ok(b),
            Err(RError::NoData) => {
                core::hint::spin_loop();
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err(RError::Timeout)
}

/// 非阻塞读取多个字节，遇到 NoData 就停止
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

/// 阻塞读取多个字节，直到填满缓冲区
pub fn read_bytes_blocking(buffer: &mut [u8]) -> Result<usize, RError> {
    let _lock = RX_MUTEX.lock();

    for (_idx, slot) in buffer.iter_mut().enumerate() {
        loop {
            match _read_byte() {
                Ok(b) => {
                    *slot = b;
                    break;
                }
                Err(RError::NoData) => {
                    core::hint::spin_loop();
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    Ok(buffer.len())
}

/// 阻塞读取一行，遇到 \n 或 \r 结束
pub fn read_line(buffer: &mut [u8]) -> Result<usize, RError> {
    let _lock = RX_MUTEX.lock();
    let mut count = 0;

    loop {
        if count >= buffer.len() {
            return Err(RError::BufferFull);
        }

        let b = loop {
            match _read_byte() {
                Ok(b) => break b,
                Err(RError::NoData) => {
                    core::hint::spin_loop();
                    continue;
                }
                Err(e) => return Err(e),
            }
        };

        match b {
            b'\n' => {
                return Ok(count);
            }
            b'\r' => loop {
                match _read_byte() {
                    Ok(b'\n') => {
                        return Ok(count);
                    }
                    Ok(next_byte) => {
                        if count < buffer.len() {
                            buffer[count] = next_byte;
                            count += 1;
                        }
                        return Ok(count);
                    }
                    Err(RError::NoData) => {
                        core::hint::spin_loop();
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            },
            _ => {
                buffer[count] = b;
                count += 1;
            }
        }
    }
}

/// 带超时的读取一行
pub fn read_line_timeout(buffer: &mut [u8], max_spins: usize) -> Result<usize, RError> {
    let _lock = RX_MUTEX.lock();
    let mut count = 0;
    let mut total_spins = 0;

    loop {
        if count >= buffer.len() {
            return Err(RError::BufferFull);
        }

        if total_spins >= max_spins {
            return Err(RError::Timeout);
        }

        let b = loop {
            match _read_byte() {
                Ok(b) => {
                    total_spins = 0;
                    break b;
                }
                Err(RError::NoData) => {
                    core::hint::spin_loop();
                    total_spins += 1;
                    if total_spins >= max_spins {
                        return Err(RError::Timeout);
                    }
                    continue;
                }
                Err(e) => return Err(e),
            }
        };

        match b {
            b'\n' => return Ok(count),
            b'\r' => loop {
                match _read_byte() {
                    Ok(b'\n') => return Ok(count),
                    Ok(next_byte) => {
                        if count < buffer.len() {
                            buffer[count] = next_byte;
                            count += 1;
                        }
                        return Ok(count);
                    }
                    Err(RError::NoData) => {
                        core::hint::spin_loop();
                        total_spins += 1;
                        if total_spins >= max_spins {
                            return Err(RError::Timeout);
                        }
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            },
            _ => {
                buffer[count] = b;
                count += 1;
            }
        }
    }
}

#[inline]
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
