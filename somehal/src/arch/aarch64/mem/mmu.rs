use core::ops::{Deref, DerefMut};

use aarch64_cpu::registers::*;
use aarch64_cpu_ext::structures::tte::{AccessPermission, Shareability, TTE4K48};
use kdef_pgtable::KLINER_OFFSET;
use log::debug;
use page_table_generic::{
    Access, MapConfig, PTEGeneric, PageTableRef, PhysAddr, TableGeneric, VirtAddr,
};
use pie_boot_loader_aarch64::CacheKind;
use spin::Mutex;

use crate::{
    arch::el::flush_tlb,
    common::{
        self,
        mem::{AccessKind, MapRangeConfig, regions_to_map},
    },
    mem::PageTable,
};

struct Allocator;

impl Access for Allocator {
    unsafe fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> Option<page_table_generic::PhysAddr> {
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        if ptr.is_null() {
            None
        } else {
            let phys = ptr as usize - KLINER_OFFSET;
            Some(phys.into())
        }
    }

    unsafe fn dealloc(&mut self, ptr: page_table_generic::PhysAddr, layout: core::alloc::Layout) {
        let ptr = (ptr.raw() + KLINER_OFFSET) as *mut u8;
        unsafe { alloc::alloc::dealloc(ptr, layout) };
    }

    fn phys_to_mut(&self, phys: page_table_generic::PhysAddr) -> *mut u8 {
        (phys.raw() + KLINER_OFFSET) as *mut u8
    }
}

#[unsafe(link_section = ".data")]
pub(crate) static KERNAL_TABLE: Mutex<Option<PageTableRef<'static, TableImpl>>> = Mutex::new(None);

impl From<AccessKind> for AccessPermission {
    fn from(value: AccessKind) -> Self {
        match value {
            AccessKind::Read => AccessPermission::ReadOnly,
            AccessKind::ReadWrite => AccessPermission::ReadWrite,
            AccessKind::ReadExecute => AccessPermission::ReadOnly,
            AccessKind::ReadWriteExecute => AccessPermission::ReadWrite,
        }
    }
}

impl From<common::mem::CacheKind> for CacheKind {
    fn from(value: common::mem::CacheKind) -> Self {
        match value {
            common::mem::CacheKind::Device => CacheKind::Device,
            common::mem::CacheKind::Normal => CacheKind::Normal,
            common::mem::CacheKind::NoCache => CacheKind::NoCache,
        }
    }
}

impl From<MapRangeConfig> for Tte {
    fn from(value: MapRangeConfig) -> Self {
        let mut tte = Tte::empty();
        tte.set_is_valid(true);
        tte.set_access();
        tte.set_shareability(if value.cpu_share {
            Shareability::InnerShareable
        } else {
            Shareability::NonShareable
        });
        tte.set_access_permission(value.access.into());
        tte.set_attr_index(CacheKind::from(value.cache).mair_idx());
        match value.access {
            AccessKind::Read | AccessKind::ReadWrite => tte.set_executable(false),
            _ => {}
        }
        tte
    }
}

pub fn new_table<'a>(
    access: &mut impl Access,
) -> Result<Table<'a>, page_table_generic::PagingError> {
    PageTableRef::create_empty(access)
}

pub(crate) fn init_mmu() {
    let mut alloc = Allocator {};
    let access = &mut alloc;
    let mut table = Table::create_empty(access).unwrap();

    for region in regions_to_map() {
        unsafe {
            debug!(
                "Map `{:<12}`: {:?} | [{:#p}, {:#p}) -> [{:#x}, {:#x})",
                region.name,
                region.access,
                region.vaddr,
                region.vaddr.add(region.size),
                region.paddr,
                region.paddr + region.size
            );

            table
                .map(
                    MapConfig::new(
                        region.vaddr.into(),
                        region.paddr.into(),
                        region.size,
                        region.into(),
                        true,
                        false,
                    ),
                    access,
                )
                .unwrap()
        };
    }
    let addr = table.paddr().raw();
    KERNAL_TABLE.lock().replace(table);

    debug!("MMU initialized with table at {addr:#x}");
    if CurrentEL.read(CurrentEL::EL) == 1 {
        TTBR1_EL1.set_baddr(addr as _);
        TTBR0_EL1.set_baddr(0);
    } else {
        TTBR0_EL2.set_baddr(addr as _);
    }
    flush_tlb(None);
}

pub fn mmap(region: MapRangeConfig) -> Result<(), page_table_generic::PagingError> {
    let mut g = KERNAL_TABLE.lock();
    let table = g.as_mut().expect("MMU not initialized");
    let mut alloc = Allocator {};
    let access = &mut alloc;
    unsafe {
        debug!(
            "Map `{:<12}`: {:?} | [{:#p}, {:#p}) -> [{:#x}, {:#x})",
            region.name,
            region.access,
            region.vaddr,
            region.vaddr.add(region.size),
            region.paddr,
            region.paddr + region.size
        );

        table.map(
            MapConfig::new(
                region.vaddr.into(),
                region.paddr.into(),
                region.size,
                region.into(),
                true,
                true,
            ),
            access,
        )
    }
}

pub fn table_map(
    table: PageTable,
    access: &mut impl Access,
    region: MapRangeConfig,
) -> Result<(), page_table_generic::PagingError> {
    let mut table: PageTableRef<'_, TableImpl> = PageTableRef::root_from_addr(table.addr.into());

    unsafe {
        debug!(
            "Map `{:<12}`: {:?} | [{:#p}, {:#p}) -> [{:#x}, {:#x})",
            region.name,
            region.access,
            region.vaddr,
            region.vaddr.add(region.size),
            region.paddr,
            region.paddr + region.size
        );

        table.map(
            MapConfig::new(
                region.vaddr.into(),
                region.paddr.into(),
                region.size,
                region.into(),
                true,
                true,
            ),
            access,
        )
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Tte(TTE4K48);

impl Tte {
    pub fn empty() -> Self {
        let mut tte = TTE4K48::new_table(0);
        tte.set_is_valid(true);
        tte.set_access();
        tte.set_shareability(Shareability::InnerShareable);

        Self(tte)
    }
}

impl Deref for Tte {
    type Target = TTE4K48;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Tte {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl core::fmt::Debug for Tte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PTE {:?}", self.paddr())
    }
}

impl PTEGeneric for Tte {
    #[inline(always)]
    fn valid(&self) -> bool {
        self.0.is_valid()
    }

    #[inline(always)]
    fn paddr(&self) -> PhysAddr {
        self.0.address().into()
    }

    #[inline(always)]
    fn set_paddr(&mut self, paddr: PhysAddr) {
        self.0.set_address(paddr.raw() as _);
    }

    #[inline(always)]
    fn set_valid(&mut self, valid: bool) {
        self.0.set_is_valid(valid);
    }

    #[inline(always)]
    fn is_huge(&self) -> bool {
        self.0.is_block()
    }

    #[inline(always)]
    fn set_is_huge(&mut self, is_block: bool) {
        if is_block {
            self.0.set_is_block();
        } else {
            self.0.set_is_table();
        }
    }
}

pub type Table<'a> = PageTableRef<'a, TableImpl>;

#[derive(Clone, Copy)]
pub struct TableImpl;

impl TableGeneric for TableImpl {
    type PTE = Tte;

    fn flush(vaddr: Option<VirtAddr>) {
        flush_tlb(vaddr.map(|o| o.raw().into()));
    }
}

pub use super::super::el::get_kernal_table;
pub use super::super::el::set_kernal_table;
