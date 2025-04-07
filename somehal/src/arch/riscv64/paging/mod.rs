mod table_s;

use core::arch::asm;

pub use table_s::*;

#[unsafe(link_section = ".data.boot.table")]
static mut BOOT_PT_SV39: [u64; 512] = [0; 512];

#[link_boot::link_boot]
mod _m {

    use core::{hint::spin_loop, ptr::slice_from_raw_parts};

    use riscv::register::satp;

    use crate::{
        ArchIf,
        arch::{Arch, entry::mmu_entry},
        dbgln,
        fdt::fdt_size,
        mem::boot::{kcode_offset, new_boot_table},
    };

    static mut IS_MMU_ENABLED: bool = false;

    #[allow(clippy::identity_op)] // (0x0 << 10) here makes sense because it's an address
    unsafe fn init_boot_page_table() {
        unsafe {
            // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
            BOOT_PT_SV39[0] = (0x0 << 10) | 0xef;
            // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
            BOOT_PT_SV39[2] = (0x80000 << 10) | 0xef;
            // 0xffff_ffc0_0000_0000..0xffff_ffc0_4000_0000, VRWX_GAD, 1G block
            BOOT_PT_SV39[0x100] = (0x0 << 10) | 0xef;
            // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
            BOOT_PT_SV39[0x102] = (0x80000 << 10) | 0xef;
        }
    }

    pub fn is_mmu_enabled() -> bool {
        unsafe { IS_MMU_ENABLED }
    }

    pub fn enable_mmu() -> ! {
        unsafe {
            let table = new_boot_table(fdt_size()).raw();

            let t1 = &*slice_from_raw_parts(table as *const usize, 512);
            let ptr1 = t1[0x102];
            dbgln!("idx1 {}", ptr1);
            let ptr2 = ptr1 >> 10 << 12;
            dbgln!("ptr2 {}", ptr2);
            let pte2 = (ptr2 as *const usize).add(1).read_volatile();
            dbgln!("pte2 {}", pte2);

            let entry = mmu_entry as *const u8 as usize + kcode_offset();

            let ppn = table >> 12;

            dbgln!("Set kernel table {}", table);
            dbgln!("pnn {}", ppn);
            dbgln!("Jump to {}", entry);

            // loop {}
            satp::set(satp::Mode::Sv39, 0, ppn);
            Arch::flush_tlb(None);

            IS_MMU_ENABLED = true;

            asm!(
                "mv  ra,  {entry}",
                "ret",
                entry = in(reg) entry,
                options(nostack, noreturn)
            )
        }
    }
}
