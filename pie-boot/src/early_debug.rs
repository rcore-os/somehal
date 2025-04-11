#[cfg(not(target_arch = "riscv64"))]
pub fn init_debugcon() -> Option<(any_uart::Uart, MemRegion)> {
    fn phys_to_virt(p: usize) -> *mut u8 {
        p as _
    }

    let fdt = get_fdt()?;
    let choson = fdt.chosen()?;
    let node = choson.debugcon()?;

    let uart = any_uart::Uart::new_by_fdt_node(&node, phys_to_virt)?;

    let reg = node.reg()?.next()?;
    let phys_start = reg.address as usize;

    Some((
        uart,
        MemRegion {
            virt_start: (phys_start + OFFSET_LINER).into(),
            size: page_size(),
            phys_start: phys_start.into(),
            name: "debug uart",
            config: MemConfig {
                access: AccessFlags::Read | AccessFlags::Write,
                cache: CacheConfig::Device,
            },
            kind: MemRegionKind::Device,
        },
    ))
}