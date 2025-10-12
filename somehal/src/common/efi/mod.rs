use uefi::prelude::*;
use uefi::println;
use uefi_raw::table::system::SystemTable;

/// EFI PE 入口点 - 符合 EFI ABI 的汇编包装
/// 参数: a0 = image_handle, a1 = system_table
#[unsafe(export_name = "efi_pe_entry")]
#[unsafe(link_section = ".text")]
pub unsafe extern "C" fn efi_pe_entry(
    image_handle: Handle,
    system_table: *const SystemTable,
) -> Status {
    unsafe {
        crate::relocate::efi_relocate();
        ::uefi::boot::set_image_handle(image_handle);
        ::uefi::table::set_system_table(system_table);
        let _ = ::uefi::helpers::init();

        println!("Hello {}", 123);
    }

    // 返回成功状态
    Status::SUCCESS
}
