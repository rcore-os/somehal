mod table_s;

use core::arch::asm;

pub use table_s::*;

#[link_boot::link_boot]
mod _m {

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
            let table = new_boot_table(fdt_size()).raw();
            let entry = mmu_entry as *const u8 as usize + kcode_offset();
            let ppn = table >> 12;

            dbgln!("Set kernel table {}", table);
            dbgln!("pnn {}", ppn);
            dbgln!("Jump to {}", entry);

            satp::set(satp::Mode::Sv39, 0, ppn);
            Arch::flush_tlb(None);

            IS_MMU_ENABLED = true;

            asm!(
                "la gp, __global_pointer$",
                "mv ra,  {entry}",
                "ret",
                entry = in(reg) entry,
                options(nostack, noreturn)
            )
        }
    }
}
