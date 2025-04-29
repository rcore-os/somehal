use std::{io::Write, path::PathBuf};

use quote::quote;

const MB: usize = 1024 * 1024;
const DEFAULT_KERNEL_STACK_SIZE: usize = 2 * MB;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(addr_bits, values(\"39\", \"48\", \"57\", \"64\"))");

    let mut addr_bits = 48usize;
    let mut page_levels = 4usize;
    let page_size = 0x1000usize;
    let stack_size = DEFAULT_KERNEL_STACK_SIZE;

    if std::env::var("CARGO_FEATURE_SV39").is_ok() {
        addr_bits = 39;
        page_levels = 3;
    }

    let addr_base: usize = !((1 << addr_bits) - 1);
    let kernel_load_vaddr = addr_base + (1 << addr_bits) / 16 * 14 + 0x200000;

    let const_content = quote! {
        pub const ADDR_BITS: usize = #addr_bits;
        pub const PAGE_LEVELS: usize = #page_levels;
        pub const KERNEL_LOAD_VADDR: usize = #kernel_load_vaddr;
        pub const PAGE_SIZE: usize = #page_size;
        pub const STACK_SIZE: usize = #stack_size;
    };

    let mut file =
        std::fs::File::create(out_dir().join("constant.rs")).expect("constant.rs create failed");
    let syntax_tree = syn::parse2(const_content).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    file.write_all(formatted.as_bytes()).unwrap();

    println!("cargo:rerun-if-changed=memory.ld");
    println!("cargo:rustc-link-search={}", out_dir().display());

    let content = include_str!("memory.ld");

    let content = content.replace("${VCODE}", &format!("{:#x}", kernel_load_vaddr));
    let content = content.replace("${KERNAL_LOAD_VMA}", &format!("{:#x}", kernel_load_vaddr));
    let content = content.replace("${PAGE_SIZE}", &format!("{:#x}", page_size));

    let mut file =
        std::fs::File::create(out_dir().join("kmem_region.x")).expect("kmem_region.x create failed");

    file.write_all(content.as_bytes())
        .expect("kmem_region.x write failed");
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}
