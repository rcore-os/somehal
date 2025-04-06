use crate::consts::KERNEL_STACK_SIZE;

/// The earliest entry point for the primary CPU.
#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot.header")]
unsafe extern "C" fn _start() -> ! {
    // a0 = hartid
    // a1 = dtb
    unsafe {
        core::arch::naked_asm!("
        mv      s0, a0                  // save hartid
        mv      s1, a1                  // save DTB pointer
        la      sp, __stack_bottom
        li      t0, {boot_stack_size}
        add     sp, sp, t0              // setup boot stack

        mv      a0, s0
        mv      a1, s1
        la      a2, {entry}
        add     a2, a2, s2
        jalr    a2                      // call rust_entry(hartid, dtb)
        j       .",
            boot_stack_size = const KERNEL_STACK_SIZE,
            entry = sym boot_entry,
        )
    }
}

#[link_boot::link_boot]
mod _m {
    use crate::dbgln;

    fn boot_entry() {
        dbgln!("Booting up");
        
    }
}
