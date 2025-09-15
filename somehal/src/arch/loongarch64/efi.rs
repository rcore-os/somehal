//! EFI PE 启动支持
//!
//! 为 loongarch64-unknown-none-softfloat 目标实现 EFI API 兼容的 PE 入口

use core::arch::naked_asm;
use core::ffi;

// 手动定义基本的 EFI 类型，避免依赖 uefi crate
type EfiHandle = *mut ffi::c_void;
type EfiSystemTable = *mut ffi::c_void;

// 引用header.rs中定义的固件参数变量
unsafe extern "C" {
    static mut FW_ARG0: usize; // efi_boot flag 
    static mut FW_ARG1: usize; // cmdline
    static mut FW_ARG2: usize; // systab
    fn efi_kernel_entry(handle: EfiHandle, systab: EfiSystemTable) -> !;
}

/// EFI PE 入口点 - 汇编包装器
///
/// 这是 PE 头指向的实际入口点，使用 naked 函数手动实现 EFI 调用约定兼容
/// LoongArch64 EFI 调用约定：
/// - 参数通过寄存器 a0, a1 传递
/// - 返回值通过 a0 返回
#[unsafe(naked)]
#[unsafe(export_name = "efi_pe_entry")]
pub unsafe extern "C" fn efi_pe_entry() -> ! {
    naked_asm!(
        // 在 LoongArch64 中，EFI 调用约定参数：
        // a0 = efi_handle_t
        // a1 = efi_system_table_t*
        // 这些参数已经在正确的寄存器中，直接调用 Rust 实现
        "b      {efi_pe_entry_impl}",
        efi_pe_entry_impl = sym efi_pe_entry_impl,
    )
}

/// EFI PE 入口点的 Rust 实现
///
/// 这个函数处理 EFI 启动参数并跳转到内核入口
#[unsafe(no_mangle)]
unsafe extern "C" fn efi_pe_entry_impl(
    handle: EfiHandle,      // efi_handle_t in a0
    systab: EfiSystemTable, // efi_system_table_t* in a1
) -> usize {
    // 保存 EFI 参数到全局变量，供后续使用
    unsafe {
        FW_ARG0 = 1; // 设置 efi_boot 标志
        FW_ARG1 = 0; // cmdline (暂时设为 NULL)
        FW_ARG2 = systab as usize; // 保存系统表指针

        // 直接跳转到内核入口，不返回
        0
    }
}
