use crate::{
    def::CacheKind,
    paging::{GB, MB, MapConfig, PageTableRef, PhysAddr, TableGeneric},
    ram::Ram,
    *,
};
use num_align::{NumAlign, NumAssertAlign};
use page_table_generic::Access;

static mut KLINER_OFFSET: usize = 0;
static mut PAGE_SIZE: usize = 0;

fn enable_mmu_el1(args: &EarlyBootArgs, fdt: usize) {
    reg::el1::setup_table_regs();
    let addr = new_boot_table::<el1::Table, _>(args, fdt, el1::Pte::new);
    reg::el1::set_table(addr.raw());
    reg::el1::setup_sctlr();
}

fn enable_mmu_el2(args: &EarlyBootArgs, fdt: usize) {
    reg::el2::setup_table_regs();
    let addr = new_boot_table::<el2::Table, _>(args, fdt, el2::Pte::new);
    reg::el2::set_table(addr.raw());
    reg::el2::setup_sctlr();
}

fn kliner_offset() -> usize {
    unsafe { KLINER_OFFSET }
}

pub(crate) fn set_page_size(size: usize) {
    unsafe {
        PAGE_SIZE = size;
    }
}

pub(crate) fn page_size() -> usize {
    unsafe { PAGE_SIZE }
}

pub fn enable_mmu(args: &EarlyBootArgs, fdt: usize) {
    unsafe {
        KLINER_OFFSET = args.kliner_offset;
        PAGE_SIZE = args.page_size;
    }
    match args.el {
        1 => enable_mmu_el1(args, fdt),
        2 => enable_mmu_el2(args, fdt),
        _ => panic!("Unsupported exception level: {}", args.el),
    }
}

/// `rsv_space` 在 `boot stack` 之后保留的空间到校
pub fn new_boot_table<T, F>(args: &EarlyBootArgs, fdt: usize, new_pte: F) -> PhysAddr
where
    T: TableGeneric,
    F: Fn(CacheKind) -> T::PTE + Copy,
{
    let kcode_offset = args.kimage_addr_vma as usize - args.kimage_addr_lma as usize;

    let mut alloc = Ram {};

    let access = &mut alloc;

    let table_start = access.current();

    printkv!("BootTable space", "[{:p} --)", table_start);

    let mut table = early_err!(PageTableRef::<'_, T>::create_empty(access));
    unsafe {
        let align = if kcode_offset.is_aligned_to(GB) {
            GB
        } else {
            2 * MB
        };

        let code_start_phys = args.kimage_addr_lma.align_down(align) as usize;

        let code_start = args.kimage_addr_vma as usize;
        let mut code_end: usize = (table_start as usize + kcode_offset).align_up(align);
        code_end = code_end.align_up(512 * MB);

        let size = (code_end - code_start).max(align);

        printkv!(
            "code",
            "[{:#x}, {:#x}) -> [{:#x}, {:#x})",
            code_start,
            code_start + size,
            code_start_phys,
            code_start_phys + size
        );

        early_err!(table.map(
            MapConfig {
                vaddr: code_start.into(),
                paddr: code_start_phys.into(),
                size,
                pte: new_pte(CacheKind::Normal),
                allow_huge: true,
                flush: false,
            },
            access,
        ));

        early_err!(add_rams(fdt, &mut table, access, new_pte));

        if debug::reg_base() > 0 {
            let paddr = debug::reg_base();
            let vaddr = paddr + KLINER_OFFSET;
            printkv!("debug", "{:#x}-> {:#x}", vaddr, paddr);
            early_err!(table.map(
                MapConfig {
                    vaddr: vaddr.into(),
                    paddr: paddr.into(),
                    size,
                    pte: new_pte(CacheKind::Device),
                    allow_huge: true,
                    flush: false,
                },
                access,
            ));
        }

        let size = if table.entry_size() == table.max_block_size() {
            table.entry_size() * (T::TABLE_LEN / 2)
        } else {
            table.max_block_size() * T::TABLE_LEN
        };
        let start = 0x0usize;

        printkv!("eq", "[{:#x}, {:#x})", start, start + size);
        #[cfg(el = "1")]
        early_err!(table.map(
            MapConfig {
                vaddr: start.into(),
                paddr: start.into(),
                size,
                pte: new_pte(CacheKind::NoCache),
                allow_huge: true,
                flush: false,
            },
            access,
        ));
    }

    let pg = table.paddr().raw() as _;
    RETURN.as_mut().pg_start = pg;
    printkv!("Table", "{pg:#p}");
    printkv!(
        "Table size",
        "{:#x}",
        access.current() as usize - table_start as usize
    );

    table.paddr()
}

fn add_rams<T, F>(
    fdt: usize,
    table: &mut PageTableRef<'_, T>,
    access: &mut impl Access,
    new_pte: F,
) -> Result<(), &'static str>
where
    T: TableGeneric,
    F: Fn(CacheKind) -> T::PTE,
{
    let fdt = match NonNull::new(fdt as _) {
        Some(v) => v,
        _ => {
            return Err("Invalid FDT pointer");
        }
    };

    let fdt: Fdt<'static> = Fdt::from_ptr(fdt).map_err(|_| "Invalid FDT pointer")?;
    for memory in fdt.memory().flat_map(|mem| mem.regions()) {
        if memory.size == 0 {
            continue; // Skip zero-sized regions
        }
        let paddr = memory.address as usize;
        let vaddr = paddr + kliner_offset();
        printkv!("ram", "{:#x}-> {:#x}", vaddr, paddr);
        unsafe {
            early_err!(table.map(
                MapConfig {
                    vaddr: vaddr.into(),
                    paddr: paddr.into(),
                    size: memory.size,
                    pte: new_pte(CacheKind::Normal),
                    allow_huge: true,
                    flush: false,
                },
                access,
            ));
        }
    }

    Ok(())
}
