use arrayvec::ArrayVec;

use crate::arch;

pub fn __print_str(s: &str) {
    arch::debug::write_bytes(s.as_bytes());
}

pub fn __print_hex(mut n: usize) {
    const HEX_BUF_SIZE: usize = 20; // 最大长度，包括前缀"0x"和数字
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

    arch::debug::write_bytes_parts(buff.into_iter());
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
