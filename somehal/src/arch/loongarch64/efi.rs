//! EFI PE 启动支持
//!
//! 为 loongarch64-unknown-none-softfloat 目标实现 EFI API 兼容的 PE 入口

use core::{
    ffi::c_void,
    ptr::null_mut,
    sync::atomic::{AtomicIsize, AtomicPtr},
};

use uefi::{helpers, prelude::*};

use crate::arch::relocate;

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

static mut IMAGE_HANDLE: uefi_raw::Handle = null_mut();
static mut SYSTEM_TABLE: *const uefi_raw::table::system::SystemTable = null_mut();
const UEFI_HELLO_MSG: &[u8] = &[
    0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0x0020, // "Hello "
    0x0045, 0x0046, 0x0049, 0x0021, // "EFI!"
    0x000D, 0x000A, // "\r\n"
    0x0000, 0x0000, // null terminator + padding
];

/// EFI PE 入口点 - 符合 EFI ABI 的汇编包装
/// 参数: a0 = image_handle, a1 = system_table
#[unsafe(export_name = "efi_pe_entry")]
#[unsafe(link_section = ".text")]
pub unsafe extern "C" fn efi_pe_entry(
    _image_handle: uefi_raw::Handle,
    system_table: *const uefi_raw::table::system::SystemTable,
) -> Status {
    unsafe {
        relocate::early_relocate();
        IMAGE_HANDLE = _image_handle;
        SYSTEM_TABLE = system_table;
        // uefi::boot::set_image_handle(_image_handle);
        // uefi::table::set_system_table(system_table.cast());

        if system_table.is_null() {
            return Status::LOAD_ERROR;
        }

        let st = &*system_table;

        if st.header.signature != uefi_raw::table::system::SystemTable::SIGNATURE {
            return Status::LOAD_ERROR;
        }
        let hello_msg: [u16; 14] = [
            0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0x0020, // "Hello "
            0x0045, 0x0046, 0x0049, 0x0021, // "EFI!"
            0x000D, 0x000A, // "\r\n"
            0x0000, 0x0000, // null terminator + padding
        ];

        let _ = ((*st.stdout).output_string)(st.stdout, hello_msg.as_ptr());
    }

    // 返回成功状态
    Status::SUCCESS
}
