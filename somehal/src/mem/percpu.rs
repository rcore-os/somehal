use core::{
    alloc::Layout,
    ptr::{NonNull, addr_of},
};

use crate::{
    handle_err,
    mem::{CPU_COUNT, page::page_size, percpu},
    mp::CpuOnArg,
    once_static::OnceStatic,
    platform::{CpuId, CpuIdx},
    println,
};

use super::{
    PERCPU_OTHER_ALL, main_memory::RegionAllocator, page::is_relocated, percpu_data_phys,
    stack_top_cpu,
};
use kmem_region::{
    IntAlign, PhysAddr,
    region::{STACK_SIZE, kcode_offset},
};
use somehal_macros::percpu_data;

#[percpu_data]
pub static mut CPU_IDX: CpuIdx = CpuIdx::new(0);
#[percpu_data]
pub static mut CPU_ID: CpuId = CpuId::new(0);

static CPU_MAP: OnceStatic<CPUMap> = OnceStatic::new();

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
        (self.ptr.raw() + if is_relocated() { kcode_offset() } else { 0 }) as *mut usize
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
    unreachable!()
}

pub fn cpu_idx_to_id(cpu_idx: CpuIdx) -> CpuId {
    let d = CPU_MAP.as_slice();
    (d[cpu_idx.raw()]).into()
}

pub fn clean() {
    let size = percpu().len();
    let ptr0 = percpu_data_phys(0.into());
    let s = unsafe { core::slice::from_raw_parts_mut(ptr0.raw() as *mut u8, size) };
    s.fill(0);

    let ptr1 = PERCPU_OTHER_ALL.addr.raw();

    let s = unsafe { core::slice::from_raw_parts_mut(ptr1 as *mut u8, PERCPU_OTHER_ALL.size) };
    s.fill(0);
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
        idx_ptr.write(0.into());
        id_ptr.write(cpu0_id);

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

                idx_ptr.write(idx.into());

                println!("write {:p} :{:?}", id_ptr, id);

                id_ptr.write(id);

                phys_iter += len;
                idx += 1;
            }
        }
    }

    unsafe { CPU_MAP.init(cpu_map) };
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
    let stack_top = stack_top_cpu(cpu_idx);

    let stack_bottom = stack_top - STACK_SIZE;

    let (table1, table2) =
        super::page::new_mapped_secondary_table(stack_bottom.raw().into(), cpu_idx);

    let arg = CpuOnArg {
        cpu_id,
        cpu_idx,
        page_table_with_liner: table1.raw().into(),
        page_table: table2.raw().into(),
    };

    let arg_addr = stack_top - size_of::<CpuOnArg>();

    unsafe {
        let arg_ptr = arg_addr.raw() as *mut CpuOnArg;
        arg_ptr.write(arg);
    }
}
