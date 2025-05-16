use core::ops::Deref;

use heap::HEAP;
use heapless::Vec;
use kmem_region::region::{
    AccessFlags, CacheConfig, MemConfig, MemRegionKind, OFFSET_LINER, STACK_SIZE, STACK_TOP,
    kcode_offset, region_phys_to_virt, region_virt_to_phys,
};
pub use kmem_region::{
    region::{KERNEL_ADDR_SPACE_SIZE, KERNEL_ADDR_SPACE_START, MemRegion},
    *,
};
use main_memory::RegionAllocator;
use rdrive::register::{DriverRegister, DriverRegisterSlice};
use somehal_macros::fn_link_section;

use crate::{
    ArchIf, CpuOnArg,
    arch::Arch,
    once_static::OnceStatic,
    platform::{self, CpuId, CpuIdx},
    printkv, println,
};

pub(crate) mod heap;
pub(crate) mod main_memory;
pub mod page;
pub(crate) mod percpu;

use page::page_size;
pub use percpu::{cpu_id_to_idx, cpu_idx_to_id};

#[derive(Debug, Clone)]
pub struct PhysMemory {
    pub addr: PhysAddr,
    pub size: usize,
}

pub type PhysMemoryArray = Vec<PhysMemory, 12>;

static MEMORY_MAIN: OnceStatic<PhysMemory> = OnceStatic::new();
static MEMORY_MAIN_ALL: OnceStatic<PhysMemory> = OnceStatic::new();
static mut CPU_COUNT: usize = 1;
static MEM_REGIONS: OnceStatic<Vec<MemRegion, 128>> = OnceStatic::new();
static STACK_ALL: OnceStatic<PhysMemory> = OnceStatic::new();

pub fn cpu_count() -> usize {
    unsafe { CPU_COUNT }
}

pub fn cpu_idx() -> CpuIdx {
    percpu::CPU_IDX.read_current()
}

pub fn cpu_id() -> CpuId {
    percpu::CPU_ID.read_current()
}

pub fn cpu_main_id() -> CpuId {
    percpu::CPU_ID.read_remote(0)
}

pub(crate) fn setup_arg(args: &CpuOnArg) {
    ::percpu::init(args.cpu_idx.raw());
    percpu::CPU_IDX.write_current(args.cpu_idx);
    percpu::CPU_ID.write_current(args.cpu_id);
}

pub(crate) fn stack_top_phys(cpu_idx: CpuIdx) -> PhysAddr {
    STACK_ALL.addr + STACK_SIZE * (cpu_idx.raw() + 1)
}
pub(crate) fn stack_top_virt(cpu_idx: CpuIdx) -> VirtAddr {
    let start = STACK_TOP - STACK_SIZE * unsafe { CPU_COUNT - 1 };
    (start + STACK_SIZE * cpu_idx.raw()).into()
}

pub(crate) fn init_heap() {
    let mut h = HEAP.lock();
    if h.size() == 0 {
        let size = MEMORY_MAIN.size / 2;
        let start = MEMORY_MAIN.addr + size;
        printkv!("Tmp heap", "[{:?}, {:?})", start, start + size);
        unsafe {
            h.init(start.raw() as _, size);
        }
    }
}

pub(crate) fn setup_memory_main(
    reserved_memories: impl Iterator<Item = PhysMemory>,
    memories: impl Iterator<Item = PhysMemory>,
    cpu_count: usize,
) {
    detect_link_space();
    unsafe {
        CPU_COUNT = cpu_count;
    };

    let text_addr = text().as_ptr() as usize - kcode_offset();

    let mut main_memory = None;

    for m in memories {
        let phys_start = m.addr;
        let phys_raw = phys_start.raw();
        let size = m.size;

        if phys_raw < text_addr && text_addr < phys_raw + m.size {
            let phys = PhysMemory {
                addr: phys_start,
                size,
            };
            main_memory = Some(phys.clone());
            unsafe { MEMORY_MAIN_ALL.set(phys) };
        } else {
            let kind = if phys_start.raw() + size < text().as_ptr() as usize {
                MemRegionKind::Reserved
            } else {
                MemRegionKind::Memory
            };

            mem_region_add(MemRegion {
                virt_start: (phys_start.raw() + OFFSET_LINER).into(),
                size,
                phys_start,
                name: "memory",
                config: MemConfig {
                    access: AccessFlags::Read | AccessFlags::Write,
                    cache: CacheConfig::Normal,
                },
                kind,
            });
        }
    }

    if let Some(mut main) = main_memory {
        let mut main_end = main.addr + main.size;

        for rsv in reserved_memories {
            let rsv_end = rsv.addr + rsv.size;

            if main.addr < rsv_end && rsv_end < main_end {
                main.addr = rsv_end.align_up(page_size());
            }
        }

        let stack_all_size = cpu_count * STACK_SIZE;

        main_end = main_end.align_down(2 * MB);
        main_end = main_end - stack_all_size;

        let stack_all = PhysMemory {
            addr: main_end,
            size: stack_all_size,
        };

        unsafe {
            STACK_ALL.set(stack_all);

            main_memory::init(main.addr, main_end);

            printkv!(
                "Found main memory",
                "[{:?}, {:?})",
                MEMORY_MAIN.addr,
                MEMORY_MAIN.addr + MEMORY_MAIN.size
            );
        }
    } else {
        println!("main memory not found!");
        panic!();
    }

    mem_region_add(MemRegion {
        virt_start: stack_top_virt(CpuIdx::primary()) - STACK_SIZE,
        size: STACK_ALL.size,
        phys_start: stack_top_phys(CpuIdx::primary()) - STACK_SIZE,
        name: "stack",
        config: MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
        kind: MemRegionKind::Stack,
    });

    init_heap();
}

pub(crate) fn setup_memory_regions(cpu0_id: CpuId, cpu_list: impl Iterator<Item = CpuId>) {
    percpu::init_percpu_data();

    let mut dyn_rodata_region = RegionAllocator::new(
        "dyn ro",
        MemConfig {
            access: AccessFlags::Read,
            cache: CacheConfig::Normal,
        },
        MemRegionKind::Reserved,
        OFFSET_LINER,
    );

    percpu::init(cpu0_id, cpu_list, &mut dyn_rodata_region);

    mem_region_add(dyn_rodata_region.into());

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

    for r in Arch::memory_regions() {
        mem_region_add(r);
    }

    for r in platform::memory_regions() {
        mem_region_add(r);
    }

    print_regions();
}

pub(crate) fn kernal_load_start_link_addr() -> usize {
    unsafe extern "C" {
        fn __kernel_load_vma();
    }

    __kernel_load_vma as _
}

fn print_regions() {
    let mut regions = MEM_REGIONS.clone();
    regions.sort_by(|a, b| a.phys_start.cmp(&b.phys_start));
    for region in regions {
        println!(
            "region {:<17}: [{:?}, {:?}) -> [{:?}, {:?}) {:?} {:?} {}",
            region.name,
            region.virt_start,
            region.virt_start + region.size,
            region.phys_start,
            region.phys_start + region.size,
            region.config,
            region.kind,
            if region.size == 0 { "skip empty" } else { "" }
        );
    }
}

fn mem_region_add(mut region: MemRegion) {
    let size = region.size.align_up(page_size());
    region.size = size;

    if size == 0 {
        return;
    }

    if unsafe { (*MEM_REGIONS.get()).as_mut().unwrap() }
        .push(region)
        .is_err()
    {
        panic!();
    }
}

fn detect_link_space() {
    let regions = Vec::new();
    unsafe {
        (*MEM_REGIONS.get()).replace(regions);
    }

    mem_region_add(link_section_to_kspace(
        ".text.boot",
        pie_boot::boot_text(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));

    mem_region_add(link_section_to_kspace(
        ".data.boot",
        pie_boot::boot_data(),
        MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
    ));

    mem_region_add(link_section_to_kspace(".text", text(), MemConfig {
        access: AccessFlags::Read | AccessFlags::Execute,
        cache: CacheConfig::Normal,
    }));
    mem_region_add(link_section_to_kspace(".rodata", rodata(), MemConfig {
        access: AccessFlags::Read | AccessFlags::Execute,
        cache: CacheConfig::Normal,
    }));
    mem_region_add(link_section_to_kspace(".rwdata", rwdata(), MemConfig {
        access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
        cache: CacheConfig::Normal,
    }));
    mem_region_add(link_section_to_kspace(".bss", bss(), MemConfig {
        access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
        cache: CacheConfig::Normal,
    }));
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

pub(crate) fn clean_bss() {
    unsafe extern "C" {
        fn __start_bss();

        fn __stop_bss();

    }
    unsafe {
        let start = __start_bss as usize;
        let stop = __stop_bss as usize;
        core::slice::from_raw_parts_mut(start as *mut u8, stop - start).fill(0);
    }
}

fn_link_section!(text);
fn_link_section!(bss);

#[inline(always)]
fn percpu() -> &'static [u8] {
    unsafe extern "C" {
        fn _percpu_load_start();

        fn _percpu_load_end();

    }
    unsafe {
        let start = _percpu_load_start as usize;
        let stop = _percpu_load_end as usize;
        core::slice::from_raw_parts(start as *const u8, stop - start)
    }
}

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

fn rodata() -> &'static [u8] {
    unsafe extern "C" {
        fn __srodata();

        fn __erodata();

    }
    unsafe {
        let start = __srodata as usize;
        let stop = __erodata as usize;
        core::slice::from_raw_parts(start as *const u8, stop - start)
    }
}

/// Returns an iterator over all physical memory regions.
pub fn memory_regions<'a>() -> impl Iterator<Item = &'a MemRegion> + 'a {
    MEM_REGIONS.iter()
}

pub fn phys_to_virt(p: PhysAddr) -> VirtAddr {
    region_phys_to_virt(memory_regions(), p)
}

pub fn virt_to_phys(v: VirtAddr) -> PhysAddr {
    region_virt_to_phys(memory_regions(), v)
}

pub fn driver_registers() -> impl Deref<Target = [DriverRegister]> {
    unsafe extern "C" {
        fn __sdriver_register();
        fn __edriver_register();
    }

    unsafe {
        let len = __edriver_register as usize - __sdriver_register as usize;

        if len == 0 {
            return DriverRegisterSlice::empty();
        }

        let data = core::slice::from_raw_parts(__sdriver_register as _, len);

        DriverRegisterSlice::from_raw(data)
    }
}
