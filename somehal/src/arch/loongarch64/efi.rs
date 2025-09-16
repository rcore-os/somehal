//! EFI PE 启动支持
//!
//! 为 loongarch64-unknown-none-softfloat 目标实现 EFI API 兼容的 PE 入口

use uefi::prelude::*;

use crate::{boot_info, common::entry::virt_entry};

const _: extern "C" fn(::uefi::Handle, *const core::ffi::c_void) -> ::uefi::Status =
    efi_main as extern "C" fn(::uefi::Handle, *const core::ffi::c_void) -> Status;

#[unsafe(export_name = "efi_pe_entry")]
pub extern "C" fn efi_main(
    internal_image_handle: ::uefi::Handle,
    internal_system_table: *const ::core::ffi::c_void,
) -> Status {
    unsafe {
        ::uefi::boot::set_image_handle(internal_image_handle);
        ::uefi::table::set_system_table(internal_system_table.cast());
    }

    uefi::helpers::init().unwrap();

    info!("Hello from loongarch64 UEFI PE!");

    boot::stall(10_000_000);

    virt_entry(boot_info());

    Status::SUCCESS
}
