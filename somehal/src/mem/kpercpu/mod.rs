use core::{alloc::Layout, ptr::NonNull};

use crate::{
    handle_err,
    mem::{CPU_COUNT, page::page_size, section_percpu},
    mp::CpuOnArg,
    once_static::OnceStatic,
    platform::{CpuId, CpuIdx},
    println,
};

pub(super) static PERCPU_ALL: OnceStatic<PhysMemory> = OnceStatic::new();

static PERCPU_DATA: OnceStatic<NonNull<[u8]>> = OnceStatic::new();

static CPU_MAP: OnceStatic<CPUMap> = OnceStatic::new();

use super::{
    PhysMemory,
    main_memory::RegionAllocator,
    mem_region_add,
    page::{BOOT_TABLE1, is_relocated},
    stack_top_phys, stack_top_virt,
};
use kmem_region::{
    IntAlign, PhysAddr,
    region::{
        AccessFlags, CacheConfig, MemConfig, MemRegion, MemRegionKind, OFFSET_LINER, PERCPU_TOP,
        kcode_offset,
    },
};

/// .
///
/// # Safety
///
/// .
pub unsafe fn percpu_data() -> NonNull<[u8]> {
    *PERCPU_DATA
}

struct CPUMap {
    ptr: PhysAddr,
    len: usize,
}

impl CPUMap {
    fn new(mut ptr: NonNull<[u8]>) -> Self {
        let len = ptr.len() / size_of::<usize>();

        Self {
            ptr: (unsafe { ptr.as_mut().as_mut_ptr() } as usize).into(),
            len,
        }
    }

    fn set(&mut self, cpu_id: CpuId, cpu_idx: usize) {
        let d = unsafe { core::slice::from_raw_parts_mut(self.ptr(), self.len) };
        d[cpu_idx] = cpu_id.raw();
    }

    fn ptr(&self) -> *mut usize {
        (self.ptr.raw() + if is_relocated() { OFFSET_LINER } else { 0 }) as *mut usize
    }

    fn as_slice(&self) -> &[usize] {
        unsafe { core::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

pub fn cpu_list() -> impl Iterator<Item = (CpuIdx, CpuId)> {
    CPU_MAP
        .as_slice()
        .iter()
        .enumerate()
        .map(|(idx, &one)| (idx.into(), one.into()))
}
pub fn cpu_id_to_idx(cpu_id: CpuId) -> CpuIdx {
    for (idx, &one) in CPU_MAP.as_slice().iter().enumerate() {
        if one == cpu_id.raw() {
            return idx.into();
        }
    }
    panic!("cpu_id_to_idx: ID [{:?}] not found", cpu_id);
}

pub fn cpu_idx_to_id(cpu_idx: CpuIdx) -> CpuId {
    let d = CPU_MAP.as_slice();
    (d[cpu_idx.raw()]).into()
}

pub fn init_percpu_data() {
    let percpu_one_size = section_percpu().len().align_up(page_size());

    println!("percpu_one_size: {}", percpu_one_size);

    let percpu_all_size = percpu_one_size * unsafe { CPU_COUNT };

    let percpu_start =
        super::main_memory::alloc(Layout::from_size_align(percpu_all_size, page_size()).unwrap());

    let percpu_all = PhysMemory {
        addr: percpu_start,
        size: percpu_all_size,
    };

    unsafe { PERCPU_ALL.set(percpu_all) };

    add_data_region();
}

fn add_data_region() {
    let end = PERCPU_TOP;
    let start = end - PERCPU_ALL.size;
    unsafe {
        PERCPU_DATA.set(NonNull::slice_from_raw_parts(
            NonNull::new_unchecked(start as _),
            PERCPU_ALL.size,
        ));
    };

    mem_region_add(MemRegion {
        virt_start: start.into(),
        size: PERCPU_ALL.size,
        phys_start: PERCPU_ALL.addr,
        name: ".percpu",
        config: MemConfig {
            access: AccessFlags::Read | AccessFlags::Write | AccessFlags::Execute,
            cache: CacheConfig::Normal,
        },
        kind: MemRegionKind::PerCpu,
    });
}

pub fn init(
    cpu0_id: CpuId,
    cpu_list: impl Iterator<Item = CpuId>,
    data_alloc: &mut RegionAllocator,
) {
    println!("Init percpu data");
    let cpu_count = unsafe { CPU_COUNT };
    let cpu_map_size = size_of::<usize>() * cpu_count;

    let cpu_map_ptr = handle_err!(
        data_alloc.allocate(Layout::from_size_align(cpu_map_size, size_of::<usize>()).unwrap())
    );

    let mut cpu_map = CPUMap::new(cpu_map_ptr);

    let len = section_percpu().len().align_up(page_size());

    unsafe extern "C" {
        fn __start_percpu();
    }

    unsafe {
        let link_start = section_percpu().as_ptr().sub(kcode_offset());

        let mut idx = 0;

        let start = PERCPU_ALL.addr.raw();
        let mut phys_iter = start;

        println!(
            "cpu {:>04} [0x{:>04x}] phys  : [{:#x}, {:#x})",
            idx,
            cpu0_id.raw(),
            phys_iter,
            phys_iter + len
        );
        cpu_map.set(cpu0_id, idx);
        core::slice::from_raw_parts_mut(phys_iter as *mut u8, len)
            .copy_from_slice(core::slice::from_raw_parts(link_start, len));

        phys_iter += len;
        idx += 1;

        for id in cpu_list {
            if id == cpu0_id {
                continue;
            }
            println!(
                "cpu {:>04} [0x{:>04x}] phys  : [{:#x}, {:#x})",
                idx,
                id.raw(),
                phys_iter,
                phys_iter + len
            );
            cpu_map.set(id, idx);

            core::slice::from_raw_parts_mut(phys_iter as *mut u8, len)
                .copy_from_slice(core::slice::from_raw_parts(link_start, len));

            phys_iter += len;
            idx += 1;
        }
    }

    unsafe { CPU_MAP.set(cpu_map) };
    println!("alloc percpu space ok");
}

pub fn setup_stack_and_table() {
    for (idx, &id) in CPU_MAP.as_slice().iter().enumerate() {
        if idx == 0 {
            continue;
        }
        setup_stack_and_table_one(idx.into(), id.into());
    }
}

fn setup_stack_and_table_one(cpu_idx: CpuIdx, cpu_id: CpuId) {
    let stack_top = stack_top_phys(cpu_idx);

    let arg = CpuOnArg {
        cpu_id,
        cpu_idx,
        boot_table: BOOT_TABLE1.raw().into(),
        stack_top_virt: stack_top_virt(cpu_idx),
    };

    let arg_addr = stack_top - size_of::<CpuOnArg>();

    unsafe {
        let arg_ptr = arg_addr.raw() as *mut CpuOnArg;
        println!("stack setup @{:p}", arg_ptr);
        println!("  cpu idx   {:?}", arg.cpu_idx);
        println!("  cpu id    {:?}", arg.cpu_id);
        println!("  stack top @{:?}", arg.stack_top_virt);

        println!("  tb1       @{:?}", arg.boot_table);

        arg_ptr.write(arg);
    }
}
