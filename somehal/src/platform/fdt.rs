use core::ptr::NonNull;

use fdt_parser::{Fdt, FdtError, Status};
use kmem_region::{
    IntAlign,
    region::{AccessFlags, CacheConfig, MemConfig, MemRegionKind, OFFSET_LINER},
};
use pie_boot::BootInfo;

use crate::{
    _alloc::*,
    mem::{
        page::{is_relocated, page_size},
        *,
    },
    once_static::OnceStatic,
    platform::CpuId,
    println,
};

static mut FDT_ADDR: usize = 0;
static mut FDT_LEN: usize = 0;
static MEM_REGION_DEBUG_CON: OnceStatic<MemRegion> = OnceStatic::new();

pub(crate) unsafe fn init(boot_info: &BootInfo) {
    unsafe {
        if let Some((ptr, len)) = boot_info.fdt {
            FDT_ADDR = ptr.as_ptr() as _;
            FDT_LEN = len;
        }
    };
}

pub fn find_memory() -> Result<PhysMemoryArray, FdtError<'static>> {
    let mut mems = PhysMemoryArray::new();

    let fdt = fdt_parser::Fdt::from_ptr(NonNull::new(fdt_ptr()).ok_or(FdtError::BadPtr)?)?;

    for mem in fdt.memory() {
        for region in mem.regions() {
            if region.size == 0 {
                continue;
            }

            if mems
                .push(PhysMemory {
                    addr: (region.address as usize).into(),
                    size: region.size,
                })
                .is_err()
            {
                println!("too many phys memory regions");
                panic!();
            };
        }
    }

    Ok(mems)
}

pub fn cpu_count() -> Result<usize, FdtError<'static>> {
    Ok(cpu_list()?.count())
}

pub fn cpu_list() -> Result<impl Iterator<Item = CpuId>, FdtError<'static>> {
    let fdt = get_fdt().ok_or(FdtError::BadPtr)?;
    let nodes = fdt.find_nodes("/cpus/cpu");
    Ok(nodes
        .filter(|node| node.name().contains("cpu@"))
        .filter(|node| !matches!(node.status(), Some(Status::Disabled)))
        .map(|node| {
            let reg = node
                .reg()
                .unwrap_or_else(|| panic!("cpu {} reg not found", node.name()))
                .next()
                .expect("cpu reg 0 not found");
            (reg.address as usize).into()
        }))
}

#[cfg(not(target_arch = "riscv64"))]
pub fn init_debugcon() -> Option<any_uart::Uart> {
    use kmem_region::region::*;

    fn phys_to_virt(p: usize) -> *mut u8 {
        p as _
    }

    let fdt = get_fdt()?;
    let choson = fdt.chosen()?;
    let node = choson.debugcon()?;

    let uart = any_uart::Uart::new_by_fdt_node(&node, phys_to_virt)?;

    let reg = node.reg()?.next()?;
    let phys_start = (reg.address as usize).align_down(page_size());

    let region = MemRegion {
        virt_start: (phys_start + OFFSET_LINER).into(),
        size: page_size(),
        phys_start: phys_start.into(),
        name: "debug uart",
        config: MemConfig {
            access: AccessFlags::Read | AccessFlags::Write,
            cache: CacheConfig::Device,
        },
        kind: MemRegionKind::Device,
    };

    unsafe { MEM_REGION_DEBUG_CON.set(region) };

    Some(uart)
}

pub fn fdt_ptr() -> *mut u8 {
    (unsafe { FDT_ADDR + if is_relocated() { OFFSET_LINER } else { 0 } }) as _
}

pub fn get_fdt<'a>() -> Option<Fdt<'a>> {
    Fdt::from_ptr(NonNull::new(fdt_ptr())?).ok()
}

pub(crate) fn memory_regions() -> vec::Vec<MemRegion> {
    unsafe {
        let mut vec = vec![];

        if FDT_ADDR != 0 && FDT_LEN != 0 {
            let start = FDT_ADDR.align_down(page_size());
            let end = (FDT_ADDR + FDT_LEN).align_up(page_size());

            vec.push(MemRegion {
                name: "fdt",
                config: MemConfig {
                    access: AccessFlags::Read,
                    cache: CacheConfig::Normal,
                },
                kind: MemRegionKind::Reserved,
                virt_start: (start + OFFSET_LINER).into(),
                size: end - start,
                phys_start: start.into(),
            });
        }

        if MEM_REGION_DEBUG_CON.is_init() {
            vec.push(MEM_REGION_DEBUG_CON.clone());
        }

        vec
    }
}

pub fn init_rdrive() {
    let fdt = fdt_ptr();
    assert!(!fdt.is_null(), "fdt addr is null");

    rdrive::init(rdrive::DriverInfoKind::Fdt {
        addr: NonNull::new(fdt).unwrap(),
    });
}
