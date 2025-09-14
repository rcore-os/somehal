use core::ptr::NonNull;

pub fn shutdown() -> ! {
    loop {}
}

pub fn init_by_fdt(fdt: Option<NonNull<u8>>) {}

pub fn cpu_on(cpu_id: usize, entry: usize)->Result<(), &'static str>{
    Ok(())
}