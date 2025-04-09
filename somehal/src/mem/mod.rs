use boot::kcode_offset;
use kmem::region::{CacheConfig, STACK_TOP, region_phys_to_virt, region_virt_to_phys};
pub use kmem::*;
use page::page_size;

use somehal_macros::fn_link_section;

pub(crate) mod boot;
pub mod page;
mod percpu;

#[derive(Debug, Clone)]
pub struct PhysMemory {
    pub addr: PhysAddr,
    pub size: usize,
}

use core::alloc::Layout;

use crate::{
    consts::KERNEL_STACK_SIZE,
    dbgln,
    platform::{CpuId, CpuIdx},
    println,
};
pub use kmem::region::MemRegion;
use kmem::region::{AccessFlags, MemConfig, MemRegionKind, OFFSET_LINER};

use crate::{once_static::OnceStatic, vec::ArrayVec};

pub type PhysMemoryArray = ArrayVec<PhysMemory, 12>;

static MEMORY_MAIN: OnceStatic<PhysMemory> = OnceStatic::new();
static mut CPU_COUNT: usize = 1;
static MEM_REGIONS: OnceStatic<ArrayVec<MemRegion, 32>> = OnceStatic::new();
static STACK_ALL: OnceStatic<PhysMemory> = OnceStatic::new();
// 除主CPU 外，其他CPU的独占Data
static PERCPU_OTHER_ALL: OnceStatic<PhysMemory> = OnceStatic::new();

pub fn cpu_idx() -> CpuIdx {
    unsafe { percpu::CPU_IDX }
}

pub fn cpu_id() -> CpuId {
    unsafe { percpu::CPU_ID }
}

pub(crate) fn stack_top_cpu0() -> PhysAddr {
    STACK_ALL.addr + KERNEL_STACK_SIZE
}

pub(crate) fn setup_memory_main(memories: impl Iterator<Item = PhysMemory>, cpu_count: usize) {
    detect_link_space();
    unsafe {
        CPU_COUNT = cpu_count;
    };
    for m in memories {
        let mut phys_start = m.addr;
        let phys_raw = phys_start.raw();
        let size = m.size;
        let mut phys_end = phys_start + size;
        let kcode_end = PhysAddr::from(link_section_end() as usize - kcode_offset());

        if phys_raw < kcode_end.raw() && kcode_end.raw() < phys_raw + m.size {
            phys_start = kcode_end;

            let stack_all_size = cpu_count * KERNEL_STACK_SIZE;

            phys_end = phys_end - stack_all_size;

            let stack_all = PhysMemory {
                addr: phys_end,
                size: stack_all_size,
            };

            unsafe {
                (*STACK_ALL.get()).replace(stack_all);

                (*MEMORY_MAIN.get()).replace(PhysMemory {
                    addr: phys_start,
                    size: phys_end.raw() - phys_start.raw(),
                });
            }
        } else {
            mem_region_add(MemRegion {
                virt_start: (phys_start.raw() + OFFSET_LINER).into(),
                size,
                phys_start,
                name: "memory",
                config: MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write,
                    cache: CacheConfig::Normal,
                },
                kind: MemRegionKind::Memory,
            });
        }
    }

    let stack_start = STACK_ALL.addr + STACK_ALL.size - KERNEL_STACK_SIZE;

    mem_region_add(MemRegion {
        virt_start: (STACK_TOP - KERNEL_STACK_SIZE).into(),
        size: KERNEL_STACK_SIZE,
        phys_start: stack_start,
        name: "stack",
        config: MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
        kind: MemRegionKind::Stack,
    });
}

pub(crate) fn setup_memory_regions(
    cpu0_id: CpuId,
    rsv: impl Iterator<Item = MemRegion>,
    cpu_list: impl Iterator<Item = CpuId>,
) {
    let percpu_one_size = percpu().len().align_up(page_size());
    let percpu_cpu0_start = percpu().as_ptr() as usize - kcode_offset();

    let cpu_other_count = unsafe { CPU_COUNT - 1 };

    let percpu_all = if cpu_other_count > 0 {
        let percpu_other_all_size = percpu_one_size * cpu_other_count;

        let percpu_start =
            main_memory_alloc(Layout::from_size_align(percpu_other_all_size, page_size()).unwrap());

        PhysMemory {
            addr: percpu_start,
            size: percpu_other_all_size,
        }
    } else {
        PhysMemory {
            addr: 0usize.into(),
            size: 0,
        }
    };

    unsafe { (*PERCPU_OTHER_ALL.get()).replace(percpu_all) };
    percpu::init(cpu0_id, cpu_list);

    mem_region_add(MemRegion {
        virt_start: percpu().as_ptr().into(),
        size: percpu_one_size,
        phys_start: percpu_cpu0_start.into(),
        name: ".percpu",
        config: MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
        kind: MemRegionKind::PerCpu,
    });

    mem_region_add(MemRegion {
        virt_start: (MEMORY_MAIN.addr.raw() + OFFSET_LINER).into(),
        size: MEMORY_MAIN.size,
        phys_start: MEMORY_MAIN.addr,
        name: "mem main",
        config: MemConfig {
            access: AccessFlags::Read | AccessFlags::Write,
            cache: CacheConfig::Normal,
        },
        kind: MemRegionKind::Memory,
    });

    for r in rsv {
        mem_region_add(r);
    }
}

pub(crate) fn kernal_load_start_link_addr() -> usize {
    BootText().as_ptr() as _
}

pub(crate) fn main_memory_alloc(layout: Layout) -> PhysAddr {
    unsafe {
        let end = MEMORY_MAIN.addr + MEMORY_MAIN.size;
        let ptr = MEMORY_MAIN.addr.align_up(layout.align());
        let start = ptr + layout.size();
        let size = end.raw() - start.raw();
        (*MEMORY_MAIN.get()).replace(PhysMemory { addr: start, size });
        ptr
    }
}

fn mem_region_add(mut region: MemRegion) {
    let size = region.size.align_up(page_size());
    region.size = size;

    println!(
        "region {:<12}: [{:?}, {:?}) -> [{:?}, {:?}) {:?} {:?} {}",
        region.name,
        region.virt_start,
        region.virt_start + region.size,
        region.phys_start,
        region.phys_start + region.size,
        region.config,
        region.kind,
        if size == 0 { "skip empty" } else { "" }
    );

    if size == 0 {
        return;
    }

    if unsafe { (*MEM_REGIONS.get()).as_mut().unwrap() }
        .try_push(region)
        .is_err()
    {
        dbgln!("MemRegion is full");
        panic!();
    }
}

fn detect_link_space() {
    let regions = ArrayVec::new();
    unsafe {
        (*MEM_REGIONS.get()).replace(regions);
    }

    mem_region_add(link_section_to_kspace(
        ".text.boot",
        BootText(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".data.boot",
        BootData(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".text",
        text(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".rodata",
        rodata(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".data",
        rwdata(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
    mem_region_add(link_section_to_kspace(
        ".bss",
        bss(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));
}

/// `section`在mmu开启前是物理地址
fn link_section_to_kspace(name: &'static str, section: &[u8], config: MemConfig) -> MemRegion {
    let virt_start: VirtAddr = section.as_ptr().into();
    let phys_start = virt_start.raw() - kcode_offset();
    let size = section.len();
    MemRegion {
        virt_start,
        size,
        name,
        phys_start: phys_start.into(),
        config,
        kind: MemRegionKind::Code,
    }
}

fn_link_section!(BootText);
fn_link_section!(BootData);
fn_link_section!(text);
fn_link_section!(rodata);
fn_link_section!(bss);
fn_link_section!(percpu);

#[inline(always)]
pub(crate) fn link_section_end() -> *const u8 {
    unsafe extern "C" {
        fn __stack_bottom();
    }
    __stack_bottom as _
}

#[inline(always)]
fn rwdata() -> &'static [u8] {
    unsafe extern "C" {
        fn __srwdata();

        fn __erwdata();

    }
    unsafe {
        let start = __srwdata as usize;
        let stop = __erwdata as usize;
        core::slice::from_raw_parts(start as *const u8, stop - start)
    }
}

/// Returns an iterator over all physical memory regions.
pub fn memory_regions() -> impl Iterator<Item = MemRegion> {
    MEM_REGIONS.clone()
}

pub fn phys_to_virt(p: PhysAddr) -> VirtAddr {
    region_phys_to_virt(memory_regions(), p)
}

pub fn virt_to_phys(v: VirtAddr) -> PhysAddr {
    region_virt_to_phys(memory_regions(), v)
}
