use core::ptr::NonNull;

use fdt_parser::Fdt;

pub fn init_debugcon(fdt: *mut u8) -> Option<()> {
    fn phys_to_virt(p: usize) -> *mut u8 {
        p as _
    }

    let uart = any_uart::init(NonNull::new(fdt)?, phys_to_virt)?;
    super::set_uart(uart)?;

    Some(())
}

pub fn fdt_size(fdt: *mut u8) -> usize {
    let ptr = match NonNull::new(fdt) {
        Some(p) => p,
        None => return 0,
    };

    let fdt = match Fdt::from_ptr(ptr) {
        Ok(fdt) => fdt,
        Err(_) => return 0,
    };

    fdt.total_size()
}
