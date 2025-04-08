mod table_s;

use core::arch::asm;

use kmem::PhysAddr;

pub use table_s::*;

#[inline(always)]
pub fn set_page_table(addr: PhysAddr) {
    unsafe { satp::set(satp::Mode::Sv48, 0, addr.raw() >> 12) };
    Arch::flush_tlb(None);
}

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
            let table = new_boot_table(fdt_size());
            let entry = mmu_entry as *const u8 as usize + kcode_offset();

            dbgln!("Set kernel table {}", table.raw());
            dbgln!("Jump to {}", entry);

            set_page_table(table);

            IS_MMU_ENABLED = true;

            asm!(
                "la a1, __global_pointer$",
                "mv  gp, a1",
                "mv ra,  {entry}",
                "ret",
                entry = in(reg) entry,
                options(nostack, noreturn)
            )
        }
    }
}
