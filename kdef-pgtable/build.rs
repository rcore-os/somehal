use std::{io::Write, path::PathBuf};

use quote::quote;
fn main() {
    println!("cargo::rustc-check-cfg=cfg(addr_bits, values(\"39\", \"48\", \"57\", \"64\"))");

    let mut va_bits = 48usize;
    let mut pg_va_bits = 48usize;
    let mut page_levels = 4usize;
    let mut page_shift = 12usize;

    let target = std::env::var("TARGET").unwrap();

    if target.contains("aarch64-") {
        va_bits = 52;
    }

    if std::env::var("CARGO_FEATURE_PG_SZ16K").is_ok() {
        page_shift = 14;
        if target.contains("aarch64-") {
            pg_va_bits = 47;
        }
    }

    if std::env::var("CARGO_FEATURE_PG_L3").is_ok() {
        va_bits = 39;
        pg_va_bits = 39;
        page_levels = 3;
    }

    let const_content = quote! {
        pub const VA_BITS: usize = #va_bits;
        pub const PG_VA_BITS: usize = #pg_va_bits;
        pub const PAGE_LEVELS: usize = #page_levels;
        pub const PAGE_SHIFT: usize = #page_shift;
    };

    let mut file =
        std::fs::File::create(out_dir().join("constant.rs")).expect("constant.rs create failed");
    let syntax_tree = syn::parse2(const_content).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    file.write_all(formatted.as_bytes()).unwrap();
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}
