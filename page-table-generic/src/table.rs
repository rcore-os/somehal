use core::{
    alloc::Layout,
    marker::PhantomData,
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
};

use crate::{
    Access, PTEGeneric, PagingError, PagingResult, PhysAddr, align::AlignTo, iter::TableIter,
};
use crate::{TableGeneric, VirtAddr};
use log::trace;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MapConfig<P: PTEGeneric> {
    pub vaddr: VirtAddr,
    pub paddr: PhysAddr,
    pub size: usize,
    pub pte: P,
    pub allow_huge: bool,
    pub flush: bool,
}

impl<P: PTEGeneric> MapConfig<P> {
    pub fn new(
        vaddr: VirtAddr,
        paddr: PhysAddr,
        size: usize,
        pte: P,
        allow_huge: bool,
        flush: bool,
    ) -> Self {
        Self {
            vaddr,
            paddr,
            size,
            pte,
            allow_huge,
            flush,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct _MapConfig<P: PTEGeneric> {
    pub vaddr: VirtAddr,
    pub paddr: PhysAddr,
    pub pte: P,
}

#[link_boot::link_boot]
mod _m {
    use crate::PTEInfo;

    #[derive(Clone, Copy)]
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

        /// Map a contiguous virtual memory region to a contiguous physical memory
        /// region with the given mapping `flags`.
        ///
        /// The virtual and physical memory regions start with `vaddr` and `paddr`
        /// respectively. The region size is `size`. The addresses and `size` must
        /// be aligned to 4K, otherwise it will return [`Err(PagingError::NotAligned)`].
        ///
        /// When `allow_huge` is true, it will try to map the region with huge pages
        /// if possible. Otherwise, it will map the region with 4K pages.
        ///
        /// [`Err(PagingError::NotAligned)`]: PagingError::NotAligned
        ///
        /// # Safety
        /// User must ensure that the physical address is valid.
        pub unsafe fn map(
            &mut self,
            config: MapConfig<T::PTE>,
            access: &mut impl Access,
        ) -> PagingResult {
            let vaddr = config.vaddr;
            let paddr = config.paddr;

            if !vaddr.raw().is_aligned_to(T::PAGE_SIZE) {
                return Err(PagingError::NotAligned("vaddr"));
            }

            if !paddr.raw().is_aligned_to(T::PAGE_SIZE) {
                return Err(PagingError::NotAligned("paddr"));
            }

            let mut size = config.size;

            let mut map_cfg = _MapConfig {
                vaddr,
                paddr,
                pte: config.pte,
            };

            while size > 0 {
                let level_deepth = if config.allow_huge {
                    self.walk
                        .detect_align_level(map_cfg.vaddr.raw(), size)
                        .min(self.walk.detect_align_level(map_cfg.paddr.raw(), size))
                } else {
                    1
                };
                unsafe { self.get_entry_or_create(map_cfg, level_deepth, access)? };

                let map_size = self.walk.copy_with_level(level_deepth).level_entry_size();

                if config.flush {
                    T::flush(Some(vaddr));
                }
                map_cfg.vaddr += map_size;
                map_cfg.paddr += map_size;
                size -= map_size;
            }
            Ok(())
        }

        pub fn iter_all<A: Access>(
            &self,
            access: &'a A,
        ) -> impl Iterator<Item = PTEInfo<T::PTE>> + 'a {
            TableIter::new(0 as _, *self, access)
        }

        pub fn release(&mut self, access: &mut impl Access) {
            self._release(0.into(), access);
            unsafe {
                access.dealloc(self.addr, Self::pte_layout());
            }
        }

        fn pte_layout() -> Layout {
            unsafe { Layout::from_size_align_unchecked(T::PAGE_SIZE, T::PAGE_SIZE) }
        }

        fn _release(&mut self, start_vaddr: VirtAddr, access: &mut impl Access) -> Option<()> {
            let start_vaddr_usize: usize = start_vaddr.raw();
            let entries = self.as_slice(access);

            if self.level() == 1 {
                return Some(());
            }

            for (i, &pte) in entries.iter().enumerate() {
                let vaddr_usize = start_vaddr_usize + i * self.entry_size();
                let vaddr = vaddr_usize.into();

                if pte.valid() {
                    let is_block = pte.is_block();

                    if self.level() > 1 && !is_block {
                        let mut table_ref = self.next_table(i, access)?;
                        table_ref._release(vaddr, access)?;

                        unsafe {
                            access.dealloc(pte.paddr(), Self::pte_layout());
                        }
                    }
                }
            }
            Some(())
        }

        unsafe fn get_entry_or_create(
            &mut self,
            map_cfg: _MapConfig<T::PTE>,
            level: usize,
            access: &mut impl Access,
        ) -> PagingResult<()> {
            let mut table = *self;
            while table.level() > 0 {
                let idx = table.index_of_table(map_cfg.vaddr);
                if table.level() == level {
                    let mut pte = map_cfg.pte;
                    pte.set_paddr(map_cfg.paddr);
                    pte.set_valid(true);
                    pte.set_is_block(level > 1);

                    table.as_slice_mut(access)[idx] = pte;
                    return Ok(());
                }
                table = unsafe { table.sub_table_or_create(idx, map_cfg, access)? };
            }
            Err(PagingError::NotAligned("vaddr"))
        }

        unsafe fn sub_table_or_create(
            &mut self,
            idx: usize,
            map_cfg: _MapConfig<T::PTE>,
            access: &mut impl Access,
        ) -> PagingResult<PageTableRef<'a, T>> {
            let mut pte = self.get_pte(idx, access);
            let sub_level = self.level() - 1;

            if pte.valid() {
                Ok(Self::from_addr(pte.paddr(), sub_level))
            } else {
                pte = map_cfg.pte;
                let table = Self::new_with_level(sub_level, access)?;
                let ptr = table.addr;
                pte.set_valid(true);
                pte.set_paddr(ptr);
                pte.set_is_block(false);

                let s = self.as_slice_mut(access);
                s[idx] = pte;

                Ok(table)
            }
        }

        fn next_table(&self, idx: usize, access: &impl Access) -> Option<Self> {
            let pte = self.get_pte(idx, access);
            if pte.is_block() {
                return None;
            }
            if pte.valid() {
                Some(Self::from_addr(pte.paddr(), self.level() - 1))
            } else {
                None
            }
        }

        pub fn as_slice(&self, access: &impl Access) -> &'a [T::PTE] {
            unsafe { &*slice_from_raw_parts(access.phys_to_mut(self.addr).cast(), T::TABLE_LEN) }
        }
        fn as_slice_mut(&mut self, access: &impl Access) -> &'a mut [T::PTE] {
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

        fn get_pte(&self, idx: usize, access: &impl Access) -> T::PTE {
            let s = self.as_slice(access);
            s[idx]
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

        fn index_of_table(&self, vaddr: VirtAddr) -> usize {
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

        fn index_of_table(&self, vaddr: VirtAddr) -> usize {
            (vaddr.raw() >> self.level_entry_size_shift()) & (Self::table_len() - 1)
        }

        fn level_entry_size(&self) -> usize {
            1 << self.level_entry_size_shift()
        }

        fn detect_align_level(&self, addr: usize, size: usize) -> usize {
            for level in (0..self.level).rev() {
                let level_size = self.copy_with_level(level).level_entry_size();
                if addr % level_size == 0 && size >= level_size {
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

        fn paddr(&self) -> PhysAddr {
            todo!()
        }

        fn set_paddr(&mut self, paddr: PhysAddr) {
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
        assert_eq!(w.index_of_table(0.into()), 0);
        assert_eq!(w.index_of_table(0x1000.into()), 1);
        assert_eq!(w.index_of_table(0x2000.into()), 2);

        let w = Walk::new(2);
        assert_eq!(w.index_of_table(0.into()), 0);
        assert_eq!(w.index_of_table((2 * MB).into()), 1);

        let w = Walk::new(3);
        assert_eq!(w.index_of_table(GB.into()), 1);

        let w = Walk::new(4);
        assert_eq!(w.index_of_table((512 * GB).into()), 1);
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
