use core::{
    alloc::Layout,
    marker::PhantomData,
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
};

use crate::{Access, PTEGeneric, PagingError, PagingResult, PhysAddr};

#[link_boot::link_boot]
mod _m {
    use crate::TableGeneric;

    pub struct PageTableRef<'a, T: TableGeneric> {
        addr: PhysAddr,
        walk: PageWalk<T>,
        _marker: PhantomData<&'a T>,
    }

    impl<'a, T: TableGeneric> PageTableRef<'a, T> {
        /// Creates a new page table reference.
        pub fn create_empty(access: &mut impl Access) -> PagingResult<Self> {
            Self::new_with_level(T::LEVEL, access)
        }
        /// New page table and returns a reference to it.
        ///
        /// `level` is level of this page, should from 1 to up.
        pub fn new_with_level(level: usize, access: &mut impl Access) -> PagingResult<Self> {
            assert!(level > 0);
            let addr = unsafe { Self::alloc_table(access)? };
            Ok(PageTableRef::from_addr(addr, level))
        }

        pub fn from_addr(addr: PhysAddr, level: usize) -> Self {
            let walk = PageWalk::new(level);
            Self {
                addr,
                walk,
                _marker: PhantomData,
            }
        }

        pub fn as_slice(&self, access: &impl Access) -> &'a [T] {
            unsafe { &*slice_from_raw_parts(access.phys_to_mut(self.addr).cast(), T::TABLE_LEN) }
        }
        fn as_slice_mut(&mut self, access: &impl Access) -> &'a mut [T] {
            unsafe {
                &mut *slice_from_raw_parts_mut(access.phys_to_mut(self.addr).cast(), T::TABLE_LEN)
            }
        }

        pub fn level(&self) -> usize {
            self.walk.level
        }

        pub fn paddr(&self) -> PhysAddr {
            self.addr
        }

        unsafe fn alloc_table(access: &mut impl Access) -> PagingResult<PhysAddr> {
            let page_size = T::PAGE_SIZE;
            let layout = unsafe { Layout::from_size_align_unchecked(page_size, page_size) };
            if let Some(addr) = unsafe { access.alloc(layout) } {
                unsafe { access.phys_to_mut(addr).write_bytes(0, page_size) };
                Ok(addr)
            } else {
                Err(PagingError::NoMemory)
            }
        }

        fn index_of_table(&self, vaddr: *const u8) -> usize {
            self.walk.index_of_table(vaddr)
        }

        // 每个页表项代表的内存大小
        pub fn entry_size(&self) -> usize {
            self.walk.level_entry_size()
        }
    }

    const fn log2(value: usize) -> usize {
        assert!(value > 0, "Value must be positive and non-zero");
        match value {
            512 => 9,
            4096 => 12,
            _ => {
                let mut v = value;
                let mut result = 0;

                // 计算最高位的位置
                while v > 1 {
                    v >>= 1; // 右移一位
                    result += 1;
                }

                result
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct PageWalk<T: TableGeneric> {
        level: usize,
        _mark: PhantomData<T>,
    }

    impl<T: TableGeneric> PageWalk<T> {
        fn new(level: usize) -> Self {
            Self {
                level,
                _mark: PhantomData,
            }
        }

        const fn table_len_pow() -> usize {
            log2(Self::table_len())
        }

        const fn page_size_pow() -> usize {
            log2(T::PAGE_SIZE)
        }

        const fn table_len() -> usize {
            T::TABLE_LEN
        }

        fn copy_with_level(&self, level: usize) -> Self {
            let mut c = *self;
            c.level = level;
            c
        }

        fn level_entry_size_shift(&self) -> usize {
            Self::page_size_pow() + (self.level - 1) * Self::table_len_pow()
        }

        fn index_of_table(&self, vaddr: *const u8) -> usize {
            (vaddr as usize >> self.level_entry_size_shift()) & (Self::table_len() - 1)
        }

        fn level_entry_size(&self) -> usize {
            1 << self.level_entry_size_shift()
        }

        fn detect_align_level(&self, vaddr: *const u8, size: usize) -> usize {
            for level in (0..self.level).rev() {
                let level_size = self.copy_with_level(level).level_entry_size();
                if vaddr as usize % level_size == 0 && size >= level_size {
                    return level;
                }
            }
            1
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;

    const MB: usize = 1024 * 1024;
    const GB: usize = 1024 * MB;

    #[test]
    fn test_log2() {
        assert_eq!(log2(512), 9);
        assert_eq!(log2(4096), 12);
    }

    #[derive(Clone, Copy)]
    struct TestTable;
    impl TableGeneric for TestTable {
        type PTE = TestPTE;

        fn flush(_vaddr: Option<crate::VirtAddr>) {
            todo!()
        }
    }

    #[derive(Clone, Copy, Debug)]
    #[repr(transparent)]
    struct TestPTE(usize);
    impl PTEGeneric for TestPTE {
        fn valid(&self) -> bool {
            todo!()
        }

        fn set_valid(&mut self, _valid: bool) {
            todo!()
        }

        fn is_block(&self) -> bool {
            todo!()
        }

        fn set_is_block(&mut self, _is_block: bool) {
            todo!()
        }
    }

    type Walk = PageWalk<TestTable>;

    #[test]
    fn test_level_entry_memory_size() {
        assert_eq!(Walk::new(1).level_entry_size(), 0x1000);
        assert_eq!(Walk::new(2).level_entry_size(), 2 * MB);
        assert_eq!(Walk::new(3).level_entry_size(), GB);
        assert_eq!(Walk::new(4).level_entry_size(), 512 * GB);
    }

    #[test]
    fn test_idx_of_table() {
        let w = Walk::new(1);
        assert_eq!(w.index_of_table(0 as _), 0);
        assert_eq!(w.index_of_table(0x1000 as _), 1);
        assert_eq!(w.index_of_table(0x2000 as _), 2);

        let w = Walk::new(2);
        assert_eq!(w.index_of_table(0 as _), 0);
        assert_eq!(w.index_of_table((2 * MB) as _), 1);

        let w = Walk::new(3);
        assert_eq!(w.index_of_table(GB as _), 1);

        let w = Walk::new(4);
        assert_eq!(w.index_of_table((512 * GB) as _), 1);
    }

    #[test]
    fn test_detect_align() {
        let s = 4 * GB;

        let w = Walk::new(4);
        assert_eq!(w.detect_align_level(0x1000 as _, s), 1);

        assert_eq!(w.detect_align_level((0x1000 * 512) as _, s), 2);

        assert_eq!(w.detect_align_level((0x1000 * 512 * 512) as _, s), 3);

        assert_eq!(w.detect_align_level((2 * GB) as _, s), 3);
    }
}
