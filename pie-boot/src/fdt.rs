use core::ptr::NonNull;

use fdt_parser::FdtHeader;

pub fn fdt_size(fdt: *mut u8) -> usize {
    let ptr = match NonNull::new(fdt) {
        Some(p) => p,
        None => return 0,
    };

    let fdt = match FdtHeader::from_ptr(ptr) {
        Ok(fdt) => fdt,
        Err(_) => return 0,
    };

    fdt.total_size()
}
