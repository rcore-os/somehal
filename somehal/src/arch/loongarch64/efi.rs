//! EFI PE 启动支持
//!
//! 为 loongarch64-unknown-none-softfloat 目标实现 EFI API 兼容的 PE 入口

use core::{arch::asm, ffi::c_void};

use uefi::prelude::*;

// EFI 系统表结构定义 (简化版本，参考 Linux kernel efistub)
#[repr(C)]
struct EfiSystemTable {
    hdr: EfiTableHeader,
    firmware_vendor: *const u16,
    firmware_revision: u32,
    console_in_handle: *const (),
    con_in: *const (),
    console_out_handle: *const (),
    con_out: *const EfiSimpleTextOutputProtocol,
    stderr_handle: *const (),
    std_err: *const (),
    runtime_services: *const (),
    boot_services: *const (),
    number_of_table_entries: usize,
    configuration_table: *const (),
}

#[repr(C)]
struct EfiTableHeader {
    signature: u64,
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
}

#[repr(C)]
struct EfiSimpleTextOutputProtocol {
    reset: *const (),
    output_string: extern "C" fn(*const EfiSimpleTextOutputProtocol, *const u16) -> Status,
    test_string: *const (),
    query_mode: *const (),
    set_mode: *const (),
    set_attribute: *const (),
    clear_screen: *const (),
    set_cursor_position: *const (),
    enable_cursor: *const (),
    mode: *const (),
}

// LoongArch64 EFI 调用实现 (参考 Linux kernel efistub)
// 使用内联汇编实现真正的 EFI ABI 调用

/// LoongArch64 EFI 调用：无参数
#[allow(dead_code)]
#[inline]
unsafe fn efi_call_0(func: *const ()) -> usize {
    let result: usize;
    unsafe {
        core::arch::asm!(
            "jirl $ra, {func}, 0",
            func = in(reg) func,
            lateout("$a0") result,
            clobber_abi("C"),
        );
    }
    result
}

/// LoongArch64 EFI 调用：1个参数
#[allow(dead_code)]
#[inline]
unsafe fn efi_call_1(func: *const (), arg1: usize) -> usize {
    let result: usize;
    unsafe {
        core::arch::asm!(
            "jirl $ra, {func}, 0",
            func = in(reg) func,
            inlateout("$a0") arg1 => result,
            clobber_abi("C"),
        );
    }
    result
}

/// LoongArch64 EFI 调用：2个参数  
#[inline]
unsafe fn efi_call_2_abi(func: *const (), arg1: usize, arg2: usize) -> usize {
    let result: usize;
    unsafe {
        core::arch::asm!(
            "jirl $ra, {func}, 0",
            func = in(reg) func,
            inlateout("$a0") arg1 => result,
            in("$a1") arg2,
            clobber_abi("C"),
        );
    }
    result
}

/// LoongArch64 EFI 调用：3个参数
#[allow(dead_code)]
#[inline]
unsafe fn efi_call_3(func: *const (), arg1: usize, arg2: usize, arg3: usize) -> usize {
    let result: usize;
    unsafe {
        core::arch::asm!(
            "jirl $ra, {func}, 0",
            func = in(reg) func,
            inlateout("$a0") arg1 => result,
            in("$a1") arg2,
            in("$a2") arg3,
            clobber_abi("C"),
        );
    }
    result
}

/// LoongArch64 EFI 调用：4个参数
#[allow(dead_code)]
#[inline]
unsafe fn efi_call_4(func: *const (), arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    let result: usize;
    unsafe {
        core::arch::asm!(
            "jirl $ra, {func}, 0",
            func = in(reg) func,
            inlateout("$a0") arg1 => result,
            in("$a1") arg2,
            in("$a2") arg3,
            in("$a3") arg4,
            clobber_abi("C"),
        );
    }
    result
}

// 便利宏，用于 EFI 调用
#[allow(unused_macros)]
macro_rules! efi_call {
    ($func:expr) => {
        unsafe { efi_call_0($func as *const ()) }
    };
    ($func:expr, $arg1:expr) => {
        unsafe { efi_call_1($func as *const (), $arg1 as usize) }
    };
    ($func:expr, $arg1:expr, $arg2:expr) => {
        unsafe { efi_call_2_abi($func as *const (), $arg1 as usize, $arg2 as usize) }
    };
    ($func:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
        unsafe {
            efi_call_3(
                $func as *const (),
                $arg1 as usize,
                $arg2 as usize,
                $arg3 as usize,
            )
        }
    };
    ($func:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {
        unsafe {
            efi_call_4(
                $func as *const (),
                $arg1 as usize,
                $arg2 as usize,
                $arg3 as usize,
                $arg4 as usize,
            )
        }
    };
}

// UTF-8 字符串转换为 UTF-16 的简单实现
fn str_to_utf16<'a>(s: &str, buffer: &'a mut [u16]) -> &'a mut [u16] {
    let mut i = 0;
    for ch in s.chars() {
        if i >= buffer.len() - 1 {
            break;
        }
        if ch as u32 <= 0xFFFF {
            buffer[i] = ch as u16;
            i += 1;
        }
    }
    buffer[i] = 0; // null terminator
    &mut buffer[..=i]
}

// LoongArch64 EFI 入口点 (参考 Linux kernel efistub)
// 使用内联汇编实现符合 EFI ABI 的入口函数

/// EFI PE 入口点 - 符合 EFI ABI 的汇编包装
/// 参数: a0 = image_handle, a1 = system_table
#[unsafe(export_name = "efi_pe_entry")]
#[unsafe(link_section = ".text")]
pub unsafe extern "C" fn efi_pe_entry(
    _image_handle: ::uefi::Handle,
    system_table: *const ::core::ffi::c_void,
) -> Status {

    // 将系统表转换为我们的结构
    unsafe {
        // 从 CSR KS1 寄存器获取 UART 基地址（类似 Linux 内核方式）
        let uart_base = csr_read(LOONGARCH_CSR_KS1);

        if uart_base != 0 {
            // 输出 Hello World 字符串
            uart_puts(
                uart_base,
                b"Hello World from Pure Rust UEFI with naked_asm!\r\n",
            );
        }
    }

    Status::DEVICE_ERROR
}

// LoongArch64 CSR 寄存器定义
const LOONGARCH_CSR_KS1: u32 = 0x31;

// UART 寄存器偏移
const UART_THR: u64 = 0x00; // Transmitter Holding Register
const UART_LSR: u64 = 0x05; // Line Status Register
const UART_LSR_THRE: u8 = 0x20; // Transmitter Holding Register Empty

// 使用内联汇编读取 CSR 寄存器
#[inline(always)]
unsafe fn csr_read(csr: u32) -> u64 {
    let value: u64;
    match csr {
        LOONGARCH_CSR_KS1 => {
            asm!(
                "csrrd {}, 0x31",
                out(reg) value,
                options(nomem, nostack, preserves_flags)
            );
        }
        _ => {
            value = 0;
        }
    }
    value
}

/// 从指定地址读取 8 位值
#[inline(always)]
unsafe fn mmio_read8(addr: u64) -> u8 {
    core::ptr::read_volatile(addr as *const u8)
}

/// 向指定地址写入 8 位值
#[inline(always)]
unsafe fn mmio_write8(addr: u64, value: u8) {
    core::ptr::write_volatile(addr as *mut u8, value);
}

/// 直接 UART 输出函数
unsafe fn uart_putc(uart_base: u64, c: u8) {
    // 等待 UART 发送器准备就绪
    while (mmio_read8(uart_base + UART_LSR) & UART_LSR_THRE) == 0 {
        // 自旋等待
    }

    // 写入字符到 UART 数据寄存器
    mmio_write8(uart_base + UART_THR, c);
}

/// 输出字符串到 UART
unsafe fn uart_puts(uart_base: u64, s: &[u8]) {
    for &c in s {
        uart_putc(uart_base, c);
    }
}
