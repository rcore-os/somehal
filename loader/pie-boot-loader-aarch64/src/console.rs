use core::fmt::Write;

#[cfg(feature = "console")]
use crate::debug::write_byte;

#[cfg(not(feature = "console"))]
pub fn write_byte(_b: u8) {}

pub fn __print_str(s: &str) {
    for &b in s.as_bytes() {
        write_byte(b);
    }
}

#[macro_export]
macro_rules! early_err {
    ($f:expr) => {
        match $f {
            Ok(v) => v,
            Err(_e) => {
                println!("{}", _e);
                loop {}
            }
        }
    };
    ($f:expr, $msg:expr) => {
        match $f {
            Ok(v) => v,
            Err(_e) => {
                println!("{}:", $msg);
                println!("{}", _e);
                loop {}
            }
        }
    };
}

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        __print_str(s);
        Ok(())
    }
}

pub fn __print(args: core::fmt::Arguments) {
    let _ = Stdout {}.write_fmt(args);
}
