use core::fmt::Write;

use spin::Mutex;

use crate::{ArchIf, arch::Arch};

pub fn __print_str(s: &str) {
    for &b in s.as_bytes() {
        Arch::early_debug_put(b);
    }
}

static TX: Mutex<()> = Mutex::new(());

struct DebugTx;
impl core::fmt::Write for DebugTx {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        __print_str(s);
        Ok(())
    }
}

pub fn write_bytes(s: &[u8]) {
    let g = TX.lock();
    for &b in s {
        Arch::early_debug_put(b);
    }
    drop(g);
}

pub fn _print(args: core::fmt::Arguments) {
    let g = TX.lock();
    let _ = DebugTx {}.write_fmt(args);
    drop(g);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::print!("{}\r\n", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! printkv {
    ($key:expr, $($arg:tt)*) => {
        $crate::print!("{:<24}: {}\r\n", $key, format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! handle_err {
    ($f:expr) => {
        match $f {
            Ok(v) => v,
            Err(e) => {
                $crate::println!("{:?}", e);
                loop {
                    use $crate::archif::ArchIf;
                    $crate::arch::Arch::wait_for_event();
                }
            }
        }
    };
    ($f:expr, $msg:expr) => {
        match $f {
            Ok(v) => v,
            Err(e) => {
                $crate::println!("{}:", $msg);
                $crate::println!("{:?}", e);
                loop {
                    use $crate::archif::ArchIf;
                    $crate::arch::Arch::wait_for_event();
                }
            }
        }
    };
}
