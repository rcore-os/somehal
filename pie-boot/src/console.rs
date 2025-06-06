use crate::paging::{PagingError, PhysAddr, VirtAddr};

use crate::{Arch, archif::ArchIf};

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

impl<T> Print for *mut T {
    fn _print(self) {
        __print_hex(self as _)
    }
}

impl Print for &str {
    fn _print(self) {
        __print_str(self);
    }
}

impl Print for PhysAddr {
    fn _print(self) {
        __print_hex(self.raw());
    }
}

impl Print for VirtAddr {
    fn _print(self) {
        __print_hex(self.raw());
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

impl Print for crate::vec::Error {
    fn _print(self) {
        match self {
            crate::vec::Error::Full => __print_str("Vec full"),
        }
    }
}

#[cfg(fdt)]
impl Print for fdt_parser::FdtError<'_> {
    fn _print(self) {
        match self {
            fdt_parser::FdtError::BadCell => __print_str("BadCell"),
            fdt_parser::FdtError::NotFound(s) => {
                __print_str("NotFound: ");
                __print_str(s);
            }
            fdt_parser::FdtError::BadMagic => __print_str("BadMagic"),
            fdt_parser::FdtError::BadPtr => __print_str("BadPtr"),
            fdt_parser::FdtError::BadCellSize(s) => {
                __print_str("BadCellSize: ");
                __print_hex(s);
            }
            fdt_parser::FdtError::Eof => __print_str("Eof"),
            fdt_parser::FdtError::MissingProperty => __print_str("MissingProperty"),
            fdt_parser::FdtError::Utf8Parse { data: _ } => __print_str("Utf8Parse"),
            fdt_parser::FdtError::FromBytesUntilNull { data: _ } => {
                __print_str("FromBytesUntilNull")
            }
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

#[macro_export]
macro_rules! early_err {
    ($f:expr) => {
        match $f {
            Ok(v) => v,
            Err(_e) => {
                $crate::dbgln!("{}", _e);
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
            Err(_e) => {
                $crate::dbgln!("{}:", $msg);
                $crate::dbgln!("{}", _e);
                loop {
                    use $crate::archif::ArchIf;
                    $crate::arch::Arch::wait_for_event();
                }
            }
        }
    };
}
