use core::ptr::addr_of;

use crate::{
    mem::{page::page_size, percpu},
    platform::{CpuId, CpuIdx},
    println,
};

use super::PERCPU_OTHER_ALL;
use kmem::{IntAlign, region::kcode_offset};
use somehal_macros::percpu_data;

#[percpu_data]
pub static mut CPU_IDX: CpuIdx = CpuIdx::new(0);
#[percpu_data]
pub static mut CPU_ID: CpuId = CpuId::new(0);

pub fn init(cpu0_id: CpuId, cpu_list: impl Iterator<Item = CpuId>) {
    println!("Init percpu data");

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
