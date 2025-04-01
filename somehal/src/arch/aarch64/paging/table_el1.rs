use core::arch::asm;

#[link_boot::link_boot]
mod _m {
    use core::fmt::Debug;

    use page_table_arm::{PTE, PTEFlags};
    use page_table_generic::{PTEGeneric, PageTableRef, TableGeneric};

    pub type TableRef<'a> = PageTableRef<'a, Table>;

    #[repr(transparent)]
    #[derive(Clone, Copy)]
    pub struct Pte(PTE);

    impl PTEGeneric for Pte {
        fn valid(&self) -> bool {
            self.0.get_flags().contains(PTEFlags::VALID)
        }

        fn paddr(&self) -> page_table_generic::PhysAddr {
            self.0.paddr().into()
        }

        fn set_paddr(&mut self, paddr: page_table_generic::PhysAddr) {
            self.0.set_paddr(paddr.raw());
        }

        fn set_valid(&mut self, valid: bool) {
            let mut v = self.0.get_flags();
            if valid {
                v.insert(PTEFlags::VALID);
            } else {
                v.remove(PTEFlags::VALID);
            }
            self.0.set_flags(v);
        }

        fn is_block(&self) -> bool {
            !self.0.get_flags().contains(PTEFlags::NON_BLOCK)
        }

        fn set_is_block(&mut self, is_block: bool) {
            let mut v = self.0.get_flags();
            if is_block {
                v.remove(PTEFlags::NON_BLOCK);
            } else {
                v.insert(PTEFlags::NON_BLOCK);
            }
            self.0.set_flags(v);
        }
    }

    impl Debug for Pte {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "PTE {:?}", self.paddr())
        }
    }

    #[derive(Clone, Copy)]
    pub struct Table;

    impl TableGeneric for Table {
        type PTE = Pte;

        fn flush(vaddr: Option<page_table_generic::VirtAddr>) {
            match vaddr {
                Some(addr) => {
                    unsafe { asm!("tlbi vaae1is, {}; dsb nsh; isb", in(reg) addr.raw()) };
                }
                None => {
                    unsafe { asm!("tlbi vmalle1is; dsb nsh; isb") };
                }
            }
        }
    }
}
