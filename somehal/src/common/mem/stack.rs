use core::ops::Range;

use crate::println;
use num_align::NumAlign;

use crate::{
    boot_info,
    common::{fdt, mem::ld::stack0},
    mem::{page_size, phys_to_virt, with_regions},
};

static mut STACK_START: usize = 0;
static mut STACK_END: usize = 0;

pub fn cpu_id_list() -> impl Iterator<Item = usize> {
    let mut start = unsafe { STACK_START };
    let end = unsafe { STACK_END };
    let len = stack0().len().align_up(page_size());
    [boot_info().cpu_id]
        .into_iter()
        .chain(core::iter::from_fn(move || {
            if start >= end {
                return None;
            }
            let id = unsafe { (phys_to_virt(start) as *const usize).read() };
            let ret = Some(id);
            start += len;
            ret
        }))
}

pub fn init_percpu_stack() -> *mut u8 {
    unsafe {
        let rsv_start = boot_info().free_memory_start;
        let mut rsv_end = rsv_start.align_up(page_size());
        STACK_START = rsv_end as _;
        let per_size = stack0().len().align_up(page_size());
        for cpu_id in fdt::cpu_id_list() {
            if cpu_id == boot_info().cpu_id {
                continue;
            }

            let start = rsv_end;
            (start as *mut usize).write(cpu_id);
            let end = start.add(per_size);
            rsv_end = end;

            println!("CPU {cpu_id:#x} stack: [{start:#p}, {end:#p})");
        }

        with_regions(|ls| {
            ls.push(crate::common::mem::MemoryRegion {
                start: rsv_start as _,
                end: rsv_end as _,
                kind: crate::common::mem::MemoryRegionKind::Reserved,
            })
            .unwrap();
        });
        STACK_END = rsv_end as _;
        rsv_end as _
    }
}

pub fn percpu_stack_range() -> Range<usize> {
    unsafe { STACK_START..STACK_END }
}

pub fn cpu_stack(cpu_id: usize) -> Range<usize> {
    unsafe {
        if cpu_id == boot_info().cpu_id {
            return stack0();
        }

        let mut start = STACK_START;

        while start < STACK_END {
            let id = (phys_to_virt(start) as *const usize).read();
            let end = start + stack0().len().align_up(page_size());
            if id == cpu_id {
                return start..end;
            }
            start = end;
        }

        panic!("CPU {cpu_id:#x} stack not found");
    }
}
