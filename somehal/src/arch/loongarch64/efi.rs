//! EFI PE 启动支持
//!
//! 为 loongarch64-unknown-none-softfloat 目标实现 EFI API 兼容的 PE 入口

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
#[unsafe(naked)]
pub unsafe extern "C" fn efi_pe_entry() -> ! {
    use core::arch::naked_asm;
    naked_asm!(
        // 保存 EFI 传入的参数
        // a0 = image_handle, a1 = system_table
        "addi.d  $sp, $sp, -32",        // 分配栈空间
        "st.d    $ra, $sp, 24",         // 保存返回地址
        "st.d    $fp, $sp, 16",         // 保存帧指针
        "addi.d  $fp, $sp, 32",         // 设置新的帧指针

        // 设置栈对齐 (16字节对齐，符合 EFI ABI)
        // 使用 li.d 和 and 指令来避免立即数范围限制
        "li.d    $t0, -16",             // 加载 -16 到临时寄存器
        "and     $t0, $sp, $t0",        // 进行按位与操作
        "move    $sp, $t0",             // 更新栈指针

        // 调用 Rust 实现的 efi_main
        // 参数已经在 a0, a1 中，符合 C 调用约定
        "bl      {efi_main}",

        // 恢复栈和返回
        "ld.d    $ra, $fp, -8",         // 恢复返回地址
        "ld.d    $fp, $fp, -16",        // 恢复帧指针
        "addi.d  $sp, $sp, 32",         // 恢复栈指针
        "jr      $ra",                  // 返回到 EFI 固件

        efi_main = sym efi_main_impl,
    )
}

/// EFI 主函数的 Rust 实现
/// 这是实际的 EFI 应用逻辑，由汇编入口调用
fn efi_main_impl(
    _image_handle: ::uefi::Handle,
    system_table: *const ::core::ffi::c_void,
) -> Status {
    // 将系统表转换为我们的结构
    let system_table = system_table as *const EfiSystemTable;

    if system_table.is_null() {
        return Status::INVALID_PARAMETER;
    }

    unsafe {
        let con_out = (*system_table).con_out;
        if con_out.is_null() {
            return Status::UNSUPPORTED;
        }

        // 准备 "Hello World\r\n" 的 UTF-16 版本
        let mut hello_utf16 = [0u16; 32];
        let hello_str = str_to_utf16("Hello World\r\n", &mut hello_utf16);

        // 使用 EFI ABI 调用 EFI ConOut->OutputString
        let output_string_func = (*con_out).output_string;
        let _result = efi_call_2_abi(
            output_string_func as *const (),
            con_out as usize,
            hello_str.as_ptr() as usize,
        );
        // EFI 返回值是 usize，直接使用

        // 也可以输出第二条消息
        let mut msg2_utf16 = [0u16; 32];
        let msg2_str = str_to_utf16("EFI Call Success!\r\n", &mut msg2_utf16);
        let _result2 = efi_call_2_abi(
            output_string_func as *const (),
            con_out as usize,
            msg2_str.as_ptr() as usize,
        );
    }

    Status::SUCCESS
}
