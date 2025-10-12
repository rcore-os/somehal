use core::ptr::{addr_of, null_mut};

use uefi_raw::*;

#[macro_use]
mod err;
#[macro_use]
mod printk;

static mut IMAGE_HANDLE: Handle = null_mut();
static mut SYSTEM_TABLE: *const table::system::SystemTable = null_mut();

// 使用 static mut 而非 const，确保通过 .data 节并需要重定位
static mut UEFI_HELLO_MSG: [u16; 14] = [
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
    image_handle: uefi_raw::Handle,
    system_table: *const uefi_raw::table::system::SystemTable,
) -> Status {
    unsafe {
        IMAGE_HANDLE = image_handle;
        SYSTEM_TABLE = system_table;

        if system_table.is_null() {
            return Status::INVALID_PARAMETER;
        }

        if systab().header.signature != uefi_raw::table::system::SystemTable::SIGNATURE {
            return Status::INVALID_PARAMETER;
        }

        let msg_ptr = core::ptr::addr_of!(UEFI_HELLO_MSG) as *const u16;
        printk::char16_puts(core::slice::from_raw_parts(msg_ptr, 14));

        const MSG: &[u16] = &[
            0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0x0020, // "Hello "
            0x0045, 0x0046, 0x0049, 0x0021, // "EFI!"
            0x000D, 0x000A, // "\r\n"
            0x0000, 0x0000, // null terminator + padding
        ];

        // 使用常量测试字符串常量是否能正常访问
        printk::char16_puts(MSG);

        let msg = [
            0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0x0020, // "Hello "
            0x0045, 0x0046, 0x0049, 0x0021, // "EFI!"
            0x000D, 0x000A, // "\r\n"
            0x0000, 0x0000, // null terminator + padding
        ];

        printk::char16_puts(&msg);

        // 使用 efi_puts 测试字符串常量是否能正常访问
        printk::efi_puts("Hello from somehal EFI application!\n");
    }

    // 返回成功状态
    Status::SUCCESS
}

fn systab() -> &'static table::system::SystemTable {
    unsafe { &*SYSTEM_TABLE }
}
