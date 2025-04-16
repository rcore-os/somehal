use core::{
    alloc::Layout,
    ptr::{NonNull, slice_from_raw_parts_mut},
};

use fdt_parser::{Fdt, FdtError};
use kmem::{IntAlign, region::MemRegionKind};

use crate::{
    mem::{PhysMemory, PhysMemoryArray, main_memory_alloc, page::page_size},
    platform::CpuId,
};

use crate::{dbgln, early_err};
use kmem::region::*;

use crate::mem::{MemRegion, boot::kcode_offset};

static mut FDT_ADDR: usize = 0;

pub fn find_memory() -> Result<PhysMemoryArray, FdtError<'static>> {
    let mut mems = PhysMemoryArray::new();

    let fdt = fdt_parser::Fdt::from_ptr(NonNull::new(fdt_ptr()).ok_or(FdtError::BadPtr)?)?;

    for mem in fdt.memory() {
        for region in mem.regions() {
            if region.size == 0 {
                continue;
            }

            if mems
                .try_push(PhysMemory {
                    addr: (region.address as usize).into(),
                    size: region.size,
                })
                .is_err()
            {
                dbgln!("too many phys memory regions");
                panic!();
            };
        }
    }

    Ok(mems)
}

pub fn cpu_count() -> Result<usize, FdtError<'static>> {
    let fdt = get_fdt().ok_or(FdtError::BadPtr)?;
    let nodes = fdt.find_nodes("/cpus/cpu");
    Ok(nodes.count())
}

pub fn cpu_list() -> Result<impl Iterator<Item = CpuId>, FdtError<'static>> {
    let fdt = get_fdt().ok_or(FdtError::BadPtr)?;
    let nodes = fdt.find_nodes("/cpus/cpu");
    Ok(nodes.map(|node| {
        let reg = node
            .reg()
            .expect("cpu reg not found")
            .next()
            .expect("cpu reg 0 not found");
        (reg.address as usize).into()
    }))
}

#[cfg(not(target_arch = "riscv64"))]
pub fn init_debugcon() -> Option<(any_uart::Uart, MemRegion)> {
    fn phys_to_virt(p: usize) -> *mut u8 {
        p as _
    }

    let fdt = get_fdt()?;
    let choson = fdt.chosen()?;
    let node = choson.debugcon()?;

    let uart = any_uart::Uart::new_by_fdt_node(&node, phys_to_virt)?;

    let reg = node.reg()?.next()?;
    let phys_start = reg.address as usize;

    Some((
        uart,
        MemRegion {
            virt_start: (phys_start + OFFSET_LINER).into(),
            size: page_size(),
            phys_start: phys_start.into(),
            name: "debug uart",
            config: MemConfig {
                access: AccessFlags::Read | AccessFlags::Write,
                cache: CacheConfig::Device,
            },
            kind: MemRegionKind::Device,
        },
    ))
}
pub(crate) unsafe fn set_fdt_ptr(fdt: *mut u8) {
    unsafe { FDT_ADDR = fdt as _ };
}

fn fdt_ptr() -> *mut u8 {
    unsafe { FDT_ADDR as _ }
}

fn get_fdt<'a>() -> Option<Fdt<'a>> {
    Fdt::from_ptr(NonNull::new(fdt_ptr())?).ok()
}

pub(crate) fn save_fdt() -> Option<MemRegion> {
    let ptr_src = fdt_ptr();
    let fdt = early_err!(Fdt::from_ptr(NonNull::new(ptr_src)?));
    let size = fdt.total_size().align_up(page_size());

    let ptr_dst =
        main_memory_alloc(Layout::from_size_align(size, page_size()).unwrap()).raw() as *mut u8;

    unsafe {
        let src = &mut *slice_from_raw_parts_mut(ptr_src, size);
        let dst = &mut *slice_from_raw_parts_mut(ptr_dst, size);
        dst.copy_from_slice(src);

        FDT_ADDR = ptr_dst as _;
    }

    Some(MemRegion {
        virt_start: (ptr_dst as usize + kcode_offset()).into(),
        size,
        phys_start: (ptr_dst as usize).into(),
        name: "fdt data",
        config: MemConfig {
            access: AccessFlags::Read,
            cache: CacheConfig::Normal,
        },
        kind: MemRegionKind::Code,
    })
}
