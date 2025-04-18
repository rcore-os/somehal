use core::arch::global_asm;

use kmem::region::KERNEL_START_VADDR;
use somehal_macros::dbgln;
use x86_64::registers::control::{Cr0Flags, Cr4Flags};
use x86_64::registers::model_specific::EferFlags;

use crate::clean_bss;
use crate::config::STACK_SIZE;

const EFER_MSR: u32 = x86::msr::IA32_EFER;

/// Flags set in the ’flags’ member of the multiboot header.
///
/// (bits 1, 16: memory information, address fields in header)
const MULTIBOOT_HEADER_FLAGS: usize = 0x0001_0002;

/// The magic field should contain this.
const MULTIBOOT_HEADER_MAGIC: usize = 0x1BADB002;

const KCODE_OFFSET: usize = KERNEL_START_VADDR - 0x200000;

/// This should be in EAX.
pub(super) const MULTIBOOT_BOOTLOADER_MAGIC: usize = 0x2BADB002;

const CR0: u64 = Cr0Flags::PROTECTED_MODE_ENABLE.bits()
    | Cr0Flags::MONITOR_COPROCESSOR.bits()
    | Cr0Flags::NUMERIC_ERROR.bits()
    | Cr0Flags::WRITE_PROTECT.bits()
    | Cr0Flags::PAGING.bits();
const CR4: u64 = Cr4Flags::PHYSICAL_ADDRESS_EXTENSION.bits()
    | Cr4Flags::PAGE_GLOBAL.bits()
    | Cr4Flags::OSFXSR.bits()
    | Cr4Flags::OSXMMEXCPT_ENABLE.bits();
const EFER: u64 = EferFlags::LONG_MODE_ENABLE.bits() | EferFlags::NO_EXECUTE_ENABLE.bits();

global_asm!(
    include_str!("multiboot.S"),
    mb_magic = const MULTIBOOT_BOOTLOADER_MAGIC,
    mb_hdr_magic = const MULTIBOOT_HEADER_MAGIC,
    mb_hdr_flags = const MULTIBOOT_HEADER_FLAGS,
    entry = sym rust_entry,
    entry_secondary = sym rust_entry_secondary,
    boot_stack_size = const STACK_SIZE,
    offset = const KCODE_OFFSET,
    cr0 = const CR0,
    cr4 = const CR4,
    efer_msr = const EFER_MSR,
    efer = const EFER,
);

fn rust_entry(magic: usize, mbi: usize) {
    unsafe {
        clean_bss();
        super::uart16550::init();

        dbgln!("\r\nBooting up");

        if magic == MULTIBOOT_BOOTLOADER_MAGIC {
            dbgln!("Multiboot {}", mbi);

            unsafe extern "C" {
                fn __vma_relocate_entry(kcode_offset: usize, mbi: usize);
            }

            __vma_relocate_entry(KCODE_OFFSET, mbi);
        }
    }
}

fn rust_entry_secondary() {}
