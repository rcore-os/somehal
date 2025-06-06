use core::ptr::NonNull;

pub fn init_debugcon(fdt: *mut u8) -> Option<()> {
    fn phys_to_virt(p: usize) -> *mut u8 {
        p as _
    }

    let uart = any_uart::init(NonNull::new(fdt)?, phys_to_virt)?;
    super::set_uart(uart)?;

    Some(())
}
