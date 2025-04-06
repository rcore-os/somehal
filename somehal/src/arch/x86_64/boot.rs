use core::arch::global_asm;

use x86::msr::IA32_EFER;
use x86_64::registers::control::{Cr0Flags, Cr4Flags};
use x86_64::registers::model_specific::EferFlags;

use crate::consts::*;

/// Flags set in the ’flags’ member of the multiboot header.
///
/// (bits 1, 16: memory information, address fields in header)
const MULTIBOOT_HEADER_FLAGS: usize = 0x0001_0002;

/// The magic field should contain this.
const MULTIBOOT_HEADER_MAGIC: usize = 0x1BADB002;

/// This should be in EAX.
pub(super) const MULTIBOOT_BOOTLOADER_MAGIC: usize = 0x2BADB002;

const KERNEL_ENTRY: usize = 0x20_0000;
const KCODE_OFFSET: usize = KERNEL_ENTRY_VADDR - KERNEL_ENTRY;

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

#[unsafe(link_section = ".bss.stack")]
static mut BOOT_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

global_asm!(
    include_str!("multiboot.S"), 
    mb_magic = const MULTIBOOT_BOOTLOADER_MAGIC,
    mb_hdr_magic = const MULTIBOOT_HEADER_MAGIC,
    mb_hdr_flags = const MULTIBOOT_HEADER_FLAGS,
    entry = sym rust_entry,
    entry_secondary = sym rust_entry_secondary,    

    offset = const KCODE_OFFSET,
    boot_stack = sym BOOT_STACK,
    boot_stack_size = const KERNEL_STACK_SIZE,

    cr0 = const CR0,
    cr4 = const CR4,
    efer_msr = const IA32_EFER,
    efer = const EFER,
);

#[link_boot::link_boot]
mod _m {
    use crate::mem::boot::clean_bss;

    unsafe extern "C" fn rust_entry(magic: usize, _mbi: usize) {
        unsafe {
            // TODO: handle multiboot info
            if magic == MULTIBOOT_BOOTLOADER_MAGIC {
                clean_bss();

                loop {}
            }
        }
    }

    #[allow(unused_variables)]
    unsafe extern "C" fn rust_entry_secondary(magic: usize) {
        if magic == MULTIBOOT_BOOTLOADER_MAGIC {
            loop {}
        }
    }
}
