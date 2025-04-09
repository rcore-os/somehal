use core::ptr::slice_from_raw_parts_mut;

use kmem::{
    GB, IntAlign, MB,
    paging::{Access, MapConfig, PhysAddr},
    region::{AccessFlags, CacheConfig, MemConfig},
};

use super::{PhysMemory, kernal_load_start_link_addr, link_section_end};
use crate::{
    ArchIf,
    arch::Arch,
    consts::KERNEL_STACK_SIZE,
    dbgln, early_err,
    mem::page::{Table, page_size},
    once_static::OnceStatic,
};

#[link_boot::link_boot]
mod _m {

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

        let code_end_phys = PhysAddr::from(link_section_end() as usize);

        let start = (code_end_phys + KERNEL_STACK_SIZE + rsv_space).align_up(page_size());

        let mut tmp_alloc = LineAllocator::new(start, GB);

        dbgln!(
            "Tmp Table space: [{}, {})",
            tmp_alloc.iter.raw(),
            tmp_alloc.end.raw()
        );

        let access = &mut tmp_alloc;

        let mut table = early_err!(Table::create_empty(access));
        unsafe {
            let align = 2 * MB;

            let code_start_phys = kernal_load_start_link_addr().align_down(align);
            let code_start = code_start_phys + kcode_offset();
            let code_end: usize = (code_end_phys + kcode_offset()).align_up(align).raw();

            let size = (code_end - code_start).max(align);

            dbgln!(
                "code : [{}, {}) -> [{}, {})",
                code_start,
                code_start + size,
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

            let size = table.entry_size() * 12;

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

    fn bss_mut() -> &'static mut [u8] {
        unsafe extern "C" {
            fn __start_bss();
            fn __stop_bss();
        }
        unsafe {
            let start = __start_bss as *mut u8;
            let end = __stop_bss as *mut u8;

            &mut *slice_from_raw_parts_mut(start, end as usize - start as usize)
        }
    }

    pub unsafe fn clean_bss() {
        bss_mut().fill(0);
    }
}
