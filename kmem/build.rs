use std::{io::Write, path::PathBuf};

use quote::quote;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(addr_bits, values(\"39\", \"48\", \"57\", \"64\"))");

    let mut addr_bits = 48usize;
    let mut page_levels = 4usize;

    if std::env::var("CARGO_FEATURE_SV39").is_ok() {
        addr_bits = 39;
        page_levels = 3;
    }

    let const_content = quote! {
        pub const ADDR_BITS: usize = #addr_bits;
        pub const PAGE_LEVELS: usize = #page_levels;
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
