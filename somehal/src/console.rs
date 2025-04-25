use core::fmt::Write;

use spin::Mutex;

use crate::{ArchIf, arch::Arch};

pub fn __print_str(s: &str) {
    write_bytes(s.as_bytes());
}

static TX: Mutex<u32> = Mutex::new(0);

struct DebugTx;
impl core::fmt::Write for DebugTx {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        __print_str(s);
        Ok(())
    }
}

pub fn write_bytes(s: &[u8]) {
    let g = TX.lock();
    Arch::early_debug_put(s);
    drop(g);
}

pub fn _print(args: core::fmt::Arguments) {
    let _ = DebugTx {}.write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    ()=>{
        $crate::console::__print_str("\r\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\r\n", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! printkv {
    ($key:expr, $($arg:tt)*) => {
        $crate::print!("{:<24}: {}\r\n", $key, format_args!($($arg)*))
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
