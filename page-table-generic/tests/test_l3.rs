use std::{
    alloc::{self, Layout},
    fmt::Debug,
};

use log::trace;
use page_table_generic::*;

bitflags::bitflags! {
    /// Page-table entry flags.
    #[derive(Debug)]
    pub struct PTEFlags: usize {
        /// Whether the PTE is valid.
        const V =   1 << 0;
        /// Whether the page is readable.
        const R =   1 << 1;
        /// Whether the page is writable.
        const W =   1 << 2;
        /// Whether the page is executable.
        const X =   1 << 3;
        /// Whether the page is accessible to user mode.
        const U =   1 << 4;
        /// Designates a global mapping.
        const G =   1 << 5;
        /// Indicates the virtual page has been read, written, or fetched from
        /// since the last time the A bit was cleared.
        const A =   1 << 6;
        /// Indicates the virtual page has been written since the last time the
        /// D bit was cleared.
        const D =   1 << 7;
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PteImpl(usize);

impl PteImpl {
    const PHYS_ADDR_MASK: usize = (1 << 54) - (1 << 10);
}

impl PTEGeneric for PteImpl {
    fn valid(&self) -> bool {
        PTEFlags::from_bits_truncate(self.0).contains(PTEFlags::V)
    }

    fn paddr(&self) -> PhysAddr {
        ((self.0 & Self::PHYS_ADDR_MASK) << 2).into()
    }

    fn set_paddr(&mut self, paddr: PhysAddr) {
        self.0 = (self.0 & !Self::PHYS_ADDR_MASK) | ((paddr.raw() >> 2) & Self::PHYS_ADDR_MASK);
    }

    fn set_valid(&mut self, valid: bool) {
        if valid {
            self.0 |= PTEFlags::V.bits();
        } else {
            self.0 &= !PTEFlags::V.bits();
        }
    }

    fn is_huge(&self) -> bool {
        PTEFlags::from_bits_truncate(self.0).intersects(PTEFlags::R | PTEFlags::W | PTEFlags::X)
    }

    fn set_is_huge(&mut self, b: bool) {
        if !b {
            self.0 &= !(PTEFlags::R | PTEFlags::W | PTEFlags::X).bits();
        }
    }
}

impl Debug for PteImpl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Pte").field(&self.0).finish()
    }
}

#[derive(Clone, Copy)]
pub struct Table;

impl TableGeneric for Table {
    type PTE = PteImpl;
    const LEVEL: usize = 3;
    const MAX_BLOCK_LEVEL: usize = 3;
    const VALID_BITS: usize = 39;

    fn flush(_vaddr: Option<VirtAddr>) {}
}

struct AccessImpl {
    used: usize,
}

impl AccessImpl {
    fn new() -> Self {
        Self { used: 0 }
    }
}

impl Access for AccessImpl {
    fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
        phys.raw() as _
    }

    unsafe fn alloc(&mut self, layout: Layout) -> Option<PhysAddr> {
        let ptr = unsafe { alloc::alloc(layout) };
        trace!("alloc: {:?}", ptr);
        self.used += layout.size();
        Some((ptr as usize).into())
    }

    unsafe fn dealloc(&mut self, ptr: PhysAddr, layout: Layout) {
        trace!("dealloc: {:?}", ptr);
        unsafe { alloc::dealloc(ptr.raw() as _, layout) };
    }
}

fn new_alloc_and_table<'a>() -> (AccessImpl, PageTableRef<'a, Table>) {
    let mut access = AccessImpl::new();
    let tb = PageTableRef::<Table>::create_empty(&mut access).unwrap();
    (access, tb)
}

#[test]
fn test_pte() {
    let mut want = PteImpl(0);
    want.set_valid(true);
    assert!(want.valid());

    let addr = PhysAddr::from(0xff123456000usize);
    want.set_paddr(addr);
    assert_eq!(want.paddr(), addr);
}

#[test]
fn test_new() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();

    let (mut access, mut pg) = new_alloc_and_table();
    unsafe {
        pg.map(
            MapConfig::new(
                0xffffff3080200000usize.into(),
                0x80200000usize.into(),
                2 * MB,
                PteImpl(0xef),
                true,
                false,
            ),
            &mut access,
        )
        .unwrap();
    }
    let msg = pg
        .as_slice(&access)
        .iter()
        .filter_map(|o| {
            if o.valid() {
                Some(format!("{:#x}", o.0))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    println!("vec: {}", msg);

    let list = pg.iter_all(&access).collect::<Vec<_>>();

    for i in &list {
        println!("l: {:x}, va: {:?} c: {:#x}", i.level, i.vaddr, i.pte.0);
    }

    assert_eq!(list.len(), 2);
}

#[test]
fn test_new2() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();

    let (mut access, mut pg) = new_alloc_and_table();
    unsafe {
        pg.map(
            MapConfig::new(
                0xffffffc080200000usize.into(),
                0x80200000usize.into(),
                0x1000,
                PteImpl(0xef),
                true,
                false,
            ),
            &mut access,
        )
        .unwrap();
    }
    let msg = pg
        .as_slice(&access)
        .iter()
        .filter_map(|o| {
            if o.valid() {
                Some(format!("{:#x}", o.0))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    println!("vec: {}", msg);

    let list = pg.iter_all(&access).collect::<Vec<_>>();

    for i in &list {
        println!("l: {:x}, va: {:?} c: {:#x}", i.level, i.vaddr, i.pte.0);
    }

    assert_eq!(list.len(), 2);
}
