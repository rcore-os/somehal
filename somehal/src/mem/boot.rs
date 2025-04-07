use core::ptr::slice_from_raw_parts_mut;

use kmem::{
    GB, IntAlign, MB,
    paging::{Access, MapConfig, PhysAddr},
    region::{AccessFlags, CacheConfig, MemConfig},
};

use super::{BootText, PhysMemory};
use crate::{
    ArchIf,
    arch::Arch,
    consts::KERNEL_STACK_SIZE,
    dbgln, early_err,
    mem::page::{Table, page_level_size, page_levels, page_size},
    once_static::OnceStatic,
};

#[link_boot::link_boot]
mod _m {
    use core::ptr::addr_of_mut;

    static mut KCODE_VA_OFFSET: usize = 0;
    static BOOT_TABLE: OnceStatic<PhysMemory> = OnceStatic::new();

    pub struct LineAllocator {
        pub start: PhysAddr,
        iter: PhysAddr,
        pub end: PhysAddr,
    }

    impl LineAllocator {
        pub fn new(start: PhysAddr, size: usize) -> Self {
            Self {
                start,
                iter: start,
                end: start + size,
            }
        }

        pub fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
            let start = self.iter.align_up(layout.align());
            if start + layout.size() > self.end {
                return None;
            }

            self.iter += layout.size().align_up(layout.align());

            Some(start)
        }

        pub fn used(&self) -> PhysMemory {
            PhysMemory {
                addr: self.start,
                size: self.iter - self.start,
            }
        }
    }

    impl Access for LineAllocator {
        unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
            LineAllocator::alloc(self, layout)
        }

        unsafe fn dealloc(&mut self, _ptr: PhysAddr, _layout: core::alloc::Layout) {}

        fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
            phys.raw() as _
        }
    }

    pub unsafe fn set_kcode_va_offset(offset: usize) {
        unsafe { KCODE_VA_OFFSET = offset };
    }

    pub fn kcode_offset() -> usize {
        unsafe { KCODE_VA_OFFSET }
    }

    /// `rsv_space` 在 `boot stack` 之后保留的空间到校
    pub fn new_boot_table(mut rsv_space: usize) -> PhysAddr {
        rsv_space = rsv_space.align_up(page_size());

        dbgln!("Rsv space: {}", rsv_space);

        let start = (link_section_end() + KERNEL_STACK_SIZE + rsv_space).align_up(page_size());

        let mut tmp_alloc = LineAllocator::new(start, GB);

        dbgln!(
            "Tmp Table space: [{}, {})",
            tmp_alloc.iter.raw(),
            tmp_alloc.end.raw()
        );

        let access = &mut tmp_alloc;

        let mut table = early_err!(Table::create_empty(access));
        unsafe {
            let code_start_phys = kernal_load_addr().align_down(page_size()).raw();
            let code_start = code_start_phys + kcode_offset();
            let code_end = (link_section_end() + kcode_offset())
                .align_up(page_size())
                .raw();
            let size = (code_end - code_start).max(2 * MB);

            dbgln!(
                "code : [{}, {}) -> [{}, {})",
                code_start,
                code_end,
                code_start_phys,
                code_start_phys + size
            );

            early_err!(table.map(
                MapConfig {
                    vaddr: code_start.into(),
                    paddr: code_start_phys.into(),
                    size,
                    pte: Arch::new_pte_with_config(MemConfig {
                        access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                        cache: CacheConfig::Normal
                    }),
                    allow_huge: true,
                    flush: false,
                },
                access,
            ));

            let size = table.entry_size() * 2;
            // let size = page_level_size(3) * 12;

            dbgln!("eq   : [{}, {})", 0usize, size);
            early_err!(table.map(
                MapConfig {
                    vaddr: 0.into(),
                    paddr: 0usize.into(),
                    size,
                    pte: Arch::new_pte_with_config(MemConfig {
                        access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
                        cache: CacheConfig::Device
                    }),
                    allow_huge: true,
                    flush: false,
                },
                access,
            ));
        }

        unsafe {
            (*BOOT_TABLE.get()).replace(tmp_alloc.used());
        }

        dbgln!("Table size: {}", BOOT_TABLE.size);

        table.paddr()
    }

    pub fn kernal_load_addr() -> PhysAddr {
        let mut addr = BootText().as_ptr() as usize;

        if Arch::is_mmu_enabled() {
            addr -= kcode_offset();
        }

        addr.into()
    }

    pub fn link_section_end() -> PhysAddr {
        unsafe extern "C" {
            static mut __stack_bottom: u8;
        }
        let addr = addr_of_mut!(__stack_bottom) as usize;

        if Arch::is_mmu_enabled() {
            addr - kcode_offset()
        } else {
            addr
        }
        .into()
    }

    fn bss_mut() -> &'static mut [u8] {
        unsafe extern "C" {
            static mut __start_bss: u8;
            static mut __stop_bss: u8;
        }
        unsafe {
            let start = addr_of_mut!(__start_bss);

            &mut *slice_from_raw_parts_mut(
                start,
                addr_of_mut!(__stop_bss) as usize - start as usize,
            )
        }
    }

    pub unsafe fn clean_bss() {
        bss_mut().fill(0);
    }
}
