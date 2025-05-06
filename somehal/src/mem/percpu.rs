use core::{
    alloc::Layout,
    ptr::{NonNull, addr_of},
};

use crate::{
    handle_err,
    mem::{CPU_COUNT, page::page_size, percpu},
    once_static::OnceStatic,
    platform::{CpuId, CpuIdx},
    println,
};

use super::{PERCPU_OTHER_ALL, main_memory::RegionAllocator};
use kmem_region::{IntAlign, region::kcode_offset};
use somehal_macros::percpu_data;

#[percpu_data]
pub static mut CPU_IDX: CpuIdx = CpuIdx::new(0);
#[percpu_data]
pub static mut CPU_ID: CpuId = CpuId::new(0);

static CPU_MAP: OnceStatic<CPUMap> = OnceStatic::new();

struct CPUMap {
    ptr: NonNull<[usize]>,
}

impl CPUMap {
    fn new(mut ptr: NonNull<[u8]>) -> Self {
        let len = ptr.len() / size_of::<usize>();

        Self {
            ptr: unsafe {
                NonNull::slice_from_raw_parts(
                    NonNull::new_unchecked(ptr.as_mut().as_mut_ptr() as *mut usize),
                    len,
                )
            },
        }
    }

    fn set(&mut self, cpu_id: CpuId, cpu_idx: usize) {
        let d = unsafe { self.ptr.as_mut() };
        d[cpu_idx] = cpu_id.raw();
    }
}

pub fn cpu_id_to_idx(cpu_id: CpuId) -> CpuIdx {
    for (idx, &one) in unsafe { CPU_MAP.ptr.as_ref().iter().enumerate() } {
        if one == cpu_id.raw() {
            return idx.into();
        }
    }
    unreachable!()
}

pub fn cpu_idx_to_id(cpu_idx: CpuIdx) -> CpuId {
    let d = unsafe { CPU_MAP.ptr.as_ref() };
    (d[cpu_idx.raw()]).into()
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

    let len = percpu().len().align_up(page_size());

    unsafe extern "C" {
        fn __start_percpu();
    }

    let link_start = __start_percpu as usize;
    let idx_offset = addr_of!(CPU_IDX) as usize - link_start;
    let id_offset = addr_of!(CPU_ID) as usize - link_start;
    unsafe {
        let cpu0_start = percpu().as_ptr().sub(kcode_offset());
        let idx_ptr = cpu0_start.add(idx_offset) as *mut CpuIdx;
        let id_ptr = cpu0_start.add(id_offset) as *mut CpuId;

        let mut idx = 0;
        idx_ptr.write_volatile(0.into());
        id_ptr.write_volatile(cpu0_id);

        println!(
            "cpu {:>04} [0x{:>04x}] phys  : [{:p}, {:p})",
            idx,
            cpu0_id.raw(),
            cpu0_start,
            cpu0_start.add(len)
        );
        cpu_map.set(cpu0_id, idx);

        if PERCPU_OTHER_ALL.size > 0 {
            let start = PERCPU_OTHER_ALL.addr.raw();
            let mut phys_iter = start;
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
                    .copy_from_slice(core::slice::from_raw_parts(cpu0_start, len));

                let idx_ptr = (phys_iter + idx_offset) as *mut CpuIdx;
                let id_ptr = (phys_iter + id_offset) as *mut CpuId;

                idx_ptr.write_volatile(idx.into());
                id_ptr.write_volatile(id);

                phys_iter += len;
                idx += 1;
            }
        }
    }

    println!("alloc percpu space ok");
}
