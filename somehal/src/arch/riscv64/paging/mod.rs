mod table_s;

use core::arch::asm;

pub use table_s::*;

#[link_boot::link_boot]
mod _m {

    use riscv::register::satp;

    use crate::{
        arch::entry::mmu_entry,
        dbgln,
        fdt::fdt_size,
        mem::boot::{kcode_offset, new_boot_table},
    };

    static mut IS_MMU_ENABLED: bool = false;

    pub fn is_mmu_enabled() -> bool {
        unsafe { IS_MMU_ENABLED }
    }

    pub fn enable_mmu() -> ! {
        unsafe {
            let table = new_boot_table(fdt_size());
            dbgln!("Set kernel table {}", table.raw());
            satp::set(satp::Mode::Sv48, 0, table.raw() >> 12);
            IS_MMU_ENABLED = true;

            riscv::asm::sfence_vma_all();

            let va = kcode_offset();

            asm!(
                "la      a2, {jump_to}",
                "add     a2, a2, {va}",
                "jalr    a2",
                "j       .",
                jump_to = sym mmu_entry,
                va = in(reg) va,
                options(nostack, noreturn)
            )
        }
    }
}
