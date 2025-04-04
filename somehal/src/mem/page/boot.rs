#[link_boot::link_boot]
mod _m {
    use kmem::{
        IntAlign, PhysAddr,
        paging::{Access, MapConfig},
    };

    use crate::{
        ArchIf,
        arch::Arch,
        dbgln, early_err,
        mem::{MEM_REGIONS, MEMORY_MAIN},
    };

    use super::{Table, page_size};

    struct Alloc {
        start: usize,
        end: usize,
    }

    impl Access for Alloc {
        unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> Option<PhysAddr> {
            let start = self.start.align_up(layout.align());
            if start + layout.size() > self.end {
                return None;
            }

            self.start += layout.size().align_up(layout.align());

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
            end: end.raw(),
        };
        dbgln!("Tmp Table space: [{}, {})", tmp_alloc.start, tmp_alloc.end);
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
}
