#![allow(unused)]

use crate::arch;
use arrayvec::ArrayVec;

const HEX_BUF_SIZE: usize = 20; // 最大长度，包括前缀"0x"和数字

pub type ConstStrList = ArrayVec<&'static str, HEX_BUF_SIZE>;

pub trait AsConstStr {
    fn to_const_str(self) -> ConstStrList;
}

impl AsConstStr for usize {
    fn to_const_str(self) -> ConstStrList {
        __hex_to_str(self)
    }
}

impl AsConstStr for u64 {
    fn to_const_str(self) -> ConstStrList {
        __hex_to_str(self as _)
    }
}

impl AsConstStr for &'static str {
    fn to_const_str(self) -> ConstStrList {
        let mut out = ArrayVec::new();
        out.push(self);
        out
    }
}

pub fn __print_str_list(list: impl IntoIterator<Item = &'static str>) {
    arch::debug::write_str_list(list.into_iter());
}

pub fn __hex_to_str(mut n: usize) -> ConstStrList {
    let mut hex_buf: [&'static str; HEX_BUF_SIZE] = ["0"; HEX_BUF_SIZE];
    let mut buff = ArrayVec::<_, HEX_BUF_SIZE>::new();
    buff.push("0x");

    if n == 0 {
        buff.push("0");
    } else {
        let mut i = 0;
        while n > 0 {
            let digit = n & 0xf;
            let ch = n_to_str(digit);
            n >>= 4; // 右移四位
            hex_buf[i] = ch;
            i += 1;
        }
        let s = &hex_buf[..i];
        for &ch in s.iter().rev() {
            buff.push(ch);
        }
    }

    buff
}

fn n_to_str(n: usize) -> &'static str {
    match n {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        0xA => "A",
        0xB => "B",
        0xC => "C",
        0xD => "D",
        0xE => "E",
        0xF => "F",
        _ => "",
    }
}
