//! EFI PE 启动支持
//!
//! 为 loongarch64-unknown-none-softfloat 目标实现 EFI API 兼容的 PE 入口

use core::ptr::null_mut;

use uefi::prelude::*;

use crate::arch::relocate;

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
const UEFI_HELLO_MSG: [u16; 14] = [
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
        // IMAGE_HANDLE = _image_handle;
        // SYSTEM_TABLE = system_table;
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

        let hello = cstr16!("Hello from somehal EFI PE entry!\r\n");

        let _ = ((*st.stdout).output_string)(st.stdout, hello_msg.as_ptr());
        let _ = ((*st.stdout).output_string)(st.stdout, UEFI_HELLO_MSG.as_ptr());
    }

    // 返回成功状态
    Status::SUCCESS
}
