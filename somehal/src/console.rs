use core::fmt::Write;

use spin::Mutex;

#[link_boot::link_boot]
mod _boot {
    use fdt_parser::FdtError;
    use kmem::paging::PagingError;

    use crate::{ArchIf, arch::Arch};

    pub trait Print {
        fn _print(self);
    }

    impl Print for usize {
        fn _print(self) {
            __print_hex(self)
        }
    }

    impl Print for u64 {
        fn _print(self) {
            __print_hex(self as _)
        }
    }

    impl Print for &str {
        fn _print(self) {
            __print_str(self);
        }
    }

    impl Print for bool {
        fn _print(self) {
            __print_str(if self { "true" } else { "false" })
        }
    }

    impl Print for PagingError {
        fn _print(self) {
            match self {
                PagingError::NoMemory => __print_str("NoMemory"),
                PagingError::NotAligned(e) => {
                    __print_str("NotAligned: ");
                    __print_str(e);
                }
                PagingError::NotMapped => __print_str("NotMapped"),
                PagingError::AlreadyMapped => __print_str("AlreadyMapped"),
            }
        }
    }

    impl Print for FdtError<'_> {
        fn _print(self) {
            match self {
                FdtError::BadCell => __print_str("BadCell"),
                FdtError::NotFound(s) => {
                    __print_str("NotFound: ");
                    __print_str(s);
                }
                FdtError::BadMagic => __print_str("BadMagic"),
                FdtError::BadPtr => __print_str("BadPtr"),
                FdtError::BadCellSize(s) => {
                    __print_str("BadCellSize: ");
                    __print_hex(s);
                }
                FdtError::Eof => __print_str("Eof"),
                FdtError::MissingProperty => __print_str("MissingProperty"),
                FdtError::Utf8Parse { data: _ } => __print_str("Utf8Parse"),
                FdtError::FromBytesUntilNull { data: _ } => __print_str("FromBytesUntilNull"),
            }
        }
    }

    pub fn __print_str(s: &str) {
        for &b in s.as_bytes() {
            Arch::early_debug_put(b);
        }
    }
    pub fn __print_hex(v: usize) {
        const HEX_BUF_SIZE: usize = 20; // 最大长度，包括前缀"0x"和数字
        let mut hex_buf: [u8; HEX_BUF_SIZE] = [b'0'; HEX_BUF_SIZE];
        let mut n = v;
        __print_str("0x");

        if n == 0 {
            __print_str("0");
            return;
        }
        let mut i = 0;
        while n > 0 {
            let digit = n & 0xf;
            let ch = if digit < 10 {
                b'0' + digit as u8
            } else {
                b'a' + (digit - 10) as u8
            };
            n >>= 4; // 右移四位
            hex_buf[i] = ch;
            i += 1;
        }
        let s = &hex_buf[..i];
        for &ch in s.iter().rev() {
            Arch::early_debug_put(ch);
        }
    }
}

#[macro_export]
macro_rules! early_err {
    ($f:expr) => {
        match $f {
            Ok(v) => v,
            Err(e) => {
                $crate::dbgln!("{}", e);
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
                $crate::dbgln!("{}:", $msg);
                $crate::dbgln!("{}", e);
                loop {
                    use $crate::archif::ArchIf;
                    $crate::arch::Arch::wait_for_event();
                }
            }
        }
    };
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
