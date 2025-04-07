mod table_s;

use core::arch::asm;

pub use table_s::*;

#[link_boot::link_boot]
mod _m {

    use core::hint::spin_loop;

    use riscv::register::satp;

    use crate::{
        ArchIf,
        arch::{Arch, entry::mmu_entry},
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

            let entry = mmu_entry as *const u8 as usize + kcode_offset();

            let pnn = table.raw();

            dbgln!("pnn1 {}", pnn);
            dbgln!("pnn2 {}", pnn >> 12);
            dbgln!("Jump to {}", entry);

            satp::set(satp::Mode::Sv39, 0, table.raw() >> 12);

            // dbgln!("open");
            // IS_MMU_ENABLED = true;

            riscv::asm::sfence_vma_all();

            asm!(
                "jalr    {entry}",
                "j       .",
                entry = in(reg) entry,
                options(nostack, noreturn)
            )
        }
    }
}
