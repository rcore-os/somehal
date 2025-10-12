use core::{
    ffi::c_void,
    ptr::{NonNull, null_mut},
};

use uefi::println;
use uefi::{prelude::*, table::system_table_raw};
use uefi_raw::table::system::SystemTable;

#[macro_use]
mod err;
#[macro_use]
mod printk;

/// EFI PE 入口点 - 符合 EFI ABI 的汇编包装
/// 参数: a0 = image_handle, a1 = system_table
#[unsafe(export_name = "efi_pe_entry")]
#[unsafe(link_section = ".text")]
pub unsafe extern "C" fn efi_pe_entry(
    image_handle: Handle,
    system_table: *const SystemTable,
) -> Status {
    unsafe {
        ::uefi::boot::set_image_handle(image_handle);
        ::uefi::table::set_system_table(system_table);
        let _ = ::uefi::helpers::init();

        // let tb = system_table_raw().expect("system table is null");

        // tb.as_mut().stdout

        // uefi::helpers::_print()

        // println!("Hello from somehal EFI application!");
        // if system_table.is_null() {
        //     return Status::INVALID_PARAMETER;
        // }

        // if systab().header.signature != uefi_raw::table::system::SystemTable::SIGNATURE {
        //     return Status::INVALID_PARAMETER;
        // }

        // 使用 efi_puts 测试字符串常量是否能正常访问
        printk::efi_puts("Hello from somehal EFI application!\n");

        crate::relocate::early_relocate();

        let a = 123;
        printk::efi_puts_fmt(format_args!("Hello {a}"));
    }

    // 返回成功状态
    Status::SUCCESS
}

fn systab() -> &'static SystemTable {
    let st = system_table_raw().expect("system table is null");
    // SAFETY: valid per requirements of `set_system_table`.
    unsafe { st.as_ref() }
}
