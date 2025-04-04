use kmem::MB;

#[link_boot::link_boot]
mod _m {
    use kmem::{
        GB, IntAlign, PhysAddr,
        paging::{Access, MapConfig},
        region::{AccessFlags, CacheConfig, MemConfig},
    };

    use crate::{
        ArchIf,
        arch::Arch,
        consts::KERNEL_STACK_SIZE,
        dbgln, early_err,
        mem::{
            MEM_REGIONS, MEMORY_MAIN, entry_addr, kcode_offset, link_section_end,
            page::{page_level_size, page_levels},
        },
    };

    use super::{Table, page_size};

    struct Alloc {
        start: usize,
        iter: usize,
        end: usize,
    }

    impl Access for Alloc {
        unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
            let start = self.iter.align_up(layout.align());
            if start + layout.size() > self.end {
                return None;
            }

            self.iter += layout.size().align_up(layout.align());

            Some(start.into())
        }

        unsafe fn dealloc(&mut self, _ptr: PhysAddr, _layout: core::alloc::Layout) {}

        fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
            phys.raw() as _
        }
    }

    pub fn new_boot_table() -> PhysAddr {
        let start = (MEMORY_MAIN.addr + MEMORY_MAIN.size / 2).align_up(page_size());
        let end = MEMORY_MAIN.addr + MEMORY_MAIN.size;
        let mut tmp_alloc = Alloc {
            start: start.raw(),
            iter: start.raw(),
            end: end.raw(),
        };
        dbgln!("Tmp Table space: [{}, {})", tmp_alloc.iter, tmp_alloc.end);
        let access = &mut tmp_alloc;

        let mut table = early_err!(Table::create_empty(access));

        for region in MEM_REGIONS.clone() {
            unsafe {
                early_err!(table.map(
                    MapConfig {
                        vaddr: region.virt_start,
                        paddr: region.phys_start,
                        size: region.size,
                        pte: Arch::new_pte_with_config(region.config),
                        allow_huge: true,
                        flush: false,
                    },
                    access,
                ));

                early_err!(table.map(
                    MapConfig {
                        vaddr: region.phys_start.raw().into(),
                        paddr: region.phys_start,
                        size: region.size,
                        pte: Arch::new_pte_with_config(region.config),
                        allow_huge: true,
                        flush: false,
                    },
                    access,
                ));
            }
        }

        table.paddr()
    }

    pub fn new_boot_table2() -> PhysAddr {
        let start = (link_section_end() + KERNEL_STACK_SIZE + 8 * MB).align_up(page_size());
        let end = start + GB;

        let mut tmp_alloc = Alloc {
            start: start.raw(),
            iter: start.raw(),
            end: end.raw(),
        };

        dbgln!("Tmp Table space: [{}, {})", tmp_alloc.iter, tmp_alloc.end);

        let access = &mut tmp_alloc;

        let mut table = early_err!(Table::create_empty(access));

        unsafe {
            let code_start_phys = entry_addr().align_down(page_size());
            let code_start = code_start_phys + kcode_offset();
            let code_end = (link_section_end() + kcode_offset())
                .align_up(page_size())
                .raw();
            let size = code_end - code_start;

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

            let size = page_level_size(page_levels());
            dbgln!("liner: [{}, {})", 0usize, size);

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

        let used = tmp_alloc.iter - tmp_alloc.start;

        dbgln!("used: {}", used);

        table.paddr()
    }
}
