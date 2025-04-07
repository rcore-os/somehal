use std::{
    alloc::{self, Layout},
    fmt::{Debug, Pointer},
    mem,
    ptr::NonNull,
};

use log::trace;
use page_table_generic::*;

use PTE::Register;
use tock_registers::{interfaces::*, register_bitfields, registers::*};

const MB: usize = 1024 * 1024;
const GB: usize = 1024 * MB;

register_bitfields! [
    u64,
    PTE [
        PA OFFSET(0) NUMBITS(48) [
        ],
        READ OFFSET(48) NUMBITS(1) [
        ],
        WRITE OFFSET(49) NUMBITS(1) [
        ],
        USER_EXECUTE OFFSET(50) NUMBITS(1) [
        ],
        USER_ACCESS OFFSET(51) NUMBITS(1) [
        ],
        PRIVILEGE_EXECUTE OFFSET(52) NUMBITS(1) [
        ],
        BLOCK OFFSET(53) NUMBITS(1) [
        ],
        CACHE OFFSET(54) NUMBITS(2) [
            NonCache = 0,
            Normal = 0b01,
            Device = 0b10,
        ],
        VALID OFFSET(63) NUMBITS(1) [

        ]
    ],
];

#[repr(transparent)]
#[derive(Clone, Copy)]
struct PteImpl(u64);

impl PteImpl {
    fn reg(&self) -> &ReadWrite<u64, PTE::Register> {
        unsafe { mem::transmute(self) }
    }
}

impl Debug for PteImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.valid() {
            return write!(f, "invalid");
        }

        write!(f, "PTE PA: {:?} Block: {:?}", self.paddr(), self.is_block())
    }
}

impl PTEGeneric for PteImpl {
    fn valid(&self) -> bool {
        self.reg().is_set(PTE::VALID)
    }

    fn paddr(&self) -> PhysAddr {
        ((self.reg().read(PTE::PA) << 12) as usize).into()
    }

    fn set_paddr(&mut self, paddr: PhysAddr) {
        let paddr = paddr.raw() >> 12;
        self.reg().modify(PTE::PA.val(paddr as _));
    }

    fn set_valid(&mut self, valid: bool) {
        self.reg().modify(if valid {
            PTE::VALID::SET
        } else {
            PTE::VALID::CLEAR
        });
    }

    fn is_block(&self) -> bool {
        self.reg().is_set(PTE::BLOCK)
    }

    fn set_is_block(&mut self, is_block: bool) {
        self.reg().modify(if is_block {
            PTE::BLOCK::SET
        } else {
            PTE::BLOCK::CLEAR
        });
    }
}

#[derive(Clone, Copy)]
struct Table;
impl TableGeneric for Table {
    type PTE = PteImpl;

    fn flush(vaddr: Option<VirtAddr>) {
        println!("flush {:?}", vaddr);
    }
}

struct AccessImpl;

impl Access for AccessImpl {
    fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8 {
        phys.raw() as _
    }

    unsafe fn alloc(&mut self, layout: Layout) -> Option<PhysAddr> {
        let ptr = unsafe { alloc::alloc(layout) };
        trace!("alloc: {:?}", ptr);
        Some((ptr as usize).into())
    }

    unsafe fn dealloc(&mut self, ptr: PhysAddr, layout: Layout) {
        trace!("dealloc: {:?}", ptr);
        unsafe { alloc::dealloc(ptr.raw() as _, layout) };
    }
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

    let mut access = AccessImpl;

    let mut pg = PageTableRef::<Table>::create_empty(&mut access).unwrap();
    unsafe {
        pg.map(
            MapConfig::new(
                0xfffff00000000000usize.into(),
                0x0000usize.into(),
                0x2000,
                PteImpl(0),
                false,
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
        println!("l: {:x}, va: {:?} c: {:?}", i.level, i.vaddr, i.pte);
    }

    assert_eq!(list.len(), 5);
}

#[test]
fn test_block() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();

    let mut access = AccessImpl;

    let mut pg = PageTableRef::<Table>::create_empty(&mut access).unwrap();

    unsafe {
        pg.map(
            MapConfig::new(
                0xff0000000000usize.into(),
                0x80000000usize.into(),
                2 * GB,
                PteImpl(0),
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
        println!("l: {:x}, va: {:?} c: {:?}", i.level, i.vaddr, i.pte);
    }

    assert_eq!(list.len(), 3);
    assert!(list.last().unwrap().pte.is_block());
}

#[test]
fn test_release() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();

    let mut access = AccessImpl;

    let mut pg = PageTableRef::<Table>::create_empty(&mut access).unwrap();

    unsafe {
        pg.map(
            MapConfig::new(
                0xffff000000000000usize.into(),
                0x0000usize.into(),
                0x2000,
                PteImpl(0),
                false,
                false,
            ),
            &mut access,
        )
        .unwrap();
    }
    for i in pg.iter_all(&access) {
        println!("l: {:x}, va: {:?} c: {:?}", i.level, i.vaddr, i.pte);
    }
    pg.release(&mut access);
}

// #[test]
// fn test_2() {
//     let _ = env_logger::builder()
//         .is_test(true)
//         .filter_level(log::LevelFilter::Trace)
//         .try_init();

//     let mut access = AccessImpl;

//     let mut pg = PageTableRef::<PteImpl>::create_empty(&mut access).unwrap();
//     unsafe {
//         pg.map_region(
//             MapConfig::new(
//                 0xffffff0000000000usize as _,
//                 0x0,
//                 AccessSetting::Read | AccessSetting::Write,
//                 CacheSetting::Device,
//             )
//             .set_user_access(AccessSetting::Read),
//             0x3b400000,
//             true,
//             &mut access,
//         )
//         .unwrap();
//         pg.map_region(
//             MapConfig::new(
//                 0xffffff007d500000usize as _,
//                 0x7d500000,
//                 AccessSetting::Read | AccessSetting::Write,
//                 CacheSetting::Device,
//             )
//             .set_user_access(AccessSetting::Read),
//             0xa000,
//             true,
//             &mut access,
//         )
//         .unwrap();
//         pg.map_region(
//             MapConfig::new(
//                 0xffffff007d500000usize as _,
//                 0x7d500000,
//                 AccessSetting::Read | AccessSetting::Write,
//                 CacheSetting::Device,
//             )
//             .set_user_access(AccessSetting::Read),
//             0xa000,
//             true,
//             &mut access,
//         )
//         .unwrap();
//     }

//     // for i in pg.iter_all(&access) {
//     //     println!("l: {:x}, va: {:#p} c: {:?}", i.level, i.vaddr, i.pte);
//     // }
//     pg.release(&mut access);
// }
