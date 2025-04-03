#[link_boot::link_boot]
mod _boot {
    use page_table_generic::PagingError;

    use crate::{ArchIf, arch::Arch};

    pub trait Print {
        fn _print(self);
    }

    impl Print for usize {
        fn _print(self) {
            hex_print(self)
        }
    }

    impl Print for u64 {
        fn _print(self) {
            hex_print(self as _)
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

    pub fn __print_str(s: &str) {
        for &b in s.as_bytes() {
            Arch::early_debug_put(b);
        }
    }
    pub fn hex_print(v: usize) {
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
                $crate::dbgln!("");
                panic!();
            }
        }
    };
    ($f:expr, $msg:expr) => {
        match $f {
            Ok(v) => v,
            Err(e) => {
                $crate::dbgln!("{}:", $msg);
                $crate::dbgln!("{}", e);
                panic!();
            }
        }
    };
}
