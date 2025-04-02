#[link_boot::link_boot]
mod _m {
    use kmem::IntAlign;
    use page_table_generic::Access;

    use super::Table;

    struct Alloc {
        start: usize,
        iter: usize,
        end: usize,
    }

    impl Access for Alloc {
        unsafe fn alloc(
            &mut self,
            layout: core::alloc::Layout,
        ) -> Option<page_table_generic::PhysAddr> {
            let start = self.iter.align_up(layout.align());
            if start + layout.size() > self.end {
                return None;
            }

            self.iter = start + layout.size();

            Some(start.into())
        }

        unsafe fn dealloc(
            &mut self,
            _ptr: page_table_generic::PhysAddr,
            _layout: core::alloc::Layout,
        ) {
        }

        fn phys_to_mut(&self, phys: page_table_generic::PhysAddr) -> *mut u8 {
            phys.raw() as _
        }
    }

    pub fn new_boot_table() {
        

        // let table = Table::create_empty(access);
    }
}
