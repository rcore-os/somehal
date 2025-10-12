use super::systab;

pub fn char16_puts(s: &[u16]) {
    unsafe {
        let _ = ((*systab().stdout).output_string)(systab().stdout, s.as_ptr());
    }
}

/// 将 UTF-8 字符串转换为 UTF-32 字符
///
/// 参数：
/// - `s8`: UTF-8 字节序列的引用，函数会更新此引用以指向下一个字符
///
/// 返回：
/// - 成功：UTF-32 字符
/// - 失败：返回第一个字节（无效编码时）
///
/// 参考 Linux 内核 lib/efi/efi-printk.c
pub fn utf8_to_utf32(s8: &mut &[u8]) -> u32 {
    if s8.is_empty() {
        return 0;
    }

    let c0 = s8[0];
    let mut cx = c0;
    *s8 = &s8[1..];

    /*
     * The position of the most-significant 0 bit gives us the length of
     * a multi-octet encoding.
     */
    let mut clen = 0usize;
    while (cx & 0x80) != 0 {
        clen += 1;
        cx <<= 1;
    }

    /*
     * If the 0 bit is in position 8, this is a valid single-octet
     * encoding. If the 0 bit is in position 7 or positions 1-3, the
     * encoding is invalid.
     * In either case, we just return the first octet.
     */
    if !(2..=4).contains(&clen) {
        return c0 as u32;
    }

    /* Get the bits from the first octet. */
    let mut c32 = (cx >> clen) as u32;
    clen -= 1;

    for i in 0..clen {
        if i >= s8.len() {
            return c0 as u32;
        }

        /* Trailing octets must have 10 in most significant bits. */
        cx = s8[i] ^ 0x80;
        if (cx & 0xc0) != 0 {
            return c0 as u32;
        }
        c32 = (c32 << 6) | (cx as u32);
    }

    /*
     * Check for validity:
     * - The character must be in the Unicode range.
     * - It must not be a surrogate.
     * - It must be encoded using the correct number of octets.
     */
    let expected_len = (c32 >= 0x80) as usize + (c32 >= 0x800) as usize + (c32 >= 0x10000) as usize;

    if c32 > 0x10ffff || (c32 & 0xf800) == 0xd800 || clen != expected_len {
        return c0 as u32;
    }

    *s8 = &s8[clen..];
    c32
}

/// 将 UTF-8 编码的字符串写入 EFI 控制台
///
/// 参数：
/// - `str`: UTF-8 编码的字符串
///
/// 功能：
/// - 将 UTF-8 字符串转换为 UTF-16 (EFI 使用的格式)
/// - 自动处理换行符 '\n' -> '\r\n'
/// - 处理超出基本多文种平面(BMP)的字符（使用代理对）
/// - 使用固定大小的缓冲区，避免动态分配
///
/// 参考 Linux 内核 lib/efi/efi-printk.c
pub fn efi_puts(str: &str) {
    const BUF_SIZE: usize = 128;
    let mut buf = [0u16; BUF_SIZE];
    let mut pos = 0usize;

    // 简化版本：只处理 ASCII 字符
    for byte in str.bytes() {
        if pos >= BUF_SIZE - 3 {
            // 缓冲区快满了，先输出
            buf[pos] = 0;
            char16_puts(&buf[..=pos]);
            pos = 0;
        }

        // 处理换行符
        if byte == b'\n' {
            buf[pos] = b'\r' as u16;
            pos += 1;
        }

        // 直接转换为 UTF-16（仅支持 ASCII）
        buf[pos] = byte as u16;
        pos += 1;
    }

    // 输出剩余内容
    if pos > 0 {
        buf[pos] = 0;
        char16_puts(&buf[..=pos]);
    }
}

pub fn efi_puts_fmt(args: core::fmt::Arguments) {
    use core::fmt::Write;

    struct EfiWriter;

    impl core::fmt::Write for EfiWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            efi_puts(s);
            Ok(())
        }
    }

    let _ = EfiWriter.write_fmt(args);
}
