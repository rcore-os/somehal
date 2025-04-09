use crate::{
    consts::{KERNEL_ENTRY_VADDR, KERNEL_STACK_SIZE},
    fdt::set_fdt_ptr,
    mem::boot::*,
};

/// The earliest entry point for the primary CPU.
#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot.header")]
unsafe extern "C" fn _start() -> ! {
    // a0 = hartid
    // a1 = dtb
    unsafe {
        core::arch::naked_asm!("
        auipc   t2, 0
        li      t0, {kernel_load_addr}
        sub     t2, t0, t2
        mv      s0, a0                  // save hartid
        mv      s1, a1                  // save DTB pointer
        la      sp, __stack_bottom
        li      t0, {boot_stack_size}
        add     sp, sp, t0              // setup boot stack

        mv      a0, s0
        mv      a1, s1
        mv      a2, t2
        call    {entry}
        ",
            kernel_load_addr = const KERNEL_ENTRY_VADDR,
            boot_stack_size = const KERNEL_STACK_SIZE,
            entry = sym boot_entry,
        )
    }
}

#[link_boot::link_boot]
mod _m {
    use crate::{arch::paging::enable_mmu, dbgln};

    static mut HART_ID: usize = 0;

    fn boot_entry(hartid: usize, fdt: *mut u8, kcode_va: usize) -> ! {
        unsafe {
            clean_bss();
            set_kcode_va_offset(kcode_va);
            set_fdt_ptr(fdt);
            HART_ID = hartid;

            dbgln!("Booting up");
            dbgln!("Entry      : {}", KERNEL_ENTRY_VADDR - kcode_va);
            dbgln!("Code offset: {}", kcode_offset());
            dbgln!("fdt        : {}", fdt);
            dbgln!("fdt size   : {}", crate::fdt::fdt_size());

            enable_mmu()
        }
    }
}

#[inline(always)]
pub fn hartid() -> usize {
    unsafe { HART_ID }
}
