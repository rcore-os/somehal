use kdef_pgtable::PAGE_SIZE;
use std::{io::Write, path::PathBuf};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(hard_float)");

    let ld_file = "loader.x";

    println!("cargo::rustc-link-arg=-T{ld_file}");
    println!("cargo::rustc-link-arg=-znostart-stop-gc");
    // println!("cargo::rustc-link-arg=-Map=target/loader.map");

    let target = std::env::var("TARGET").unwrap();

    if target.as_str() == "aarch64-unknown-none" {
        println!("cargo::rustc-cfg=hard_float");
    }

    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rustc-link-search={}", out_dir().display());

    let mut content = include_str!("link.ld").to_string();
    content = content.replace("{PAGE_SIZE}", &format!("{PAGE_SIZE:#x}"));
    let mut file = std::fs::File::create(out_dir().join(ld_file)).expect("ld create failed");
    file.write_all(content.as_bytes()).expect("ld write failed");

    println!("cargo::rustc-check-cfg=cfg(el, values(\"1\", \"2\", \"3\"))");
    if std::env::var("CARGO_FEATURE_EL3").is_ok() {
        println!("cargo::rustc-cfg=el=\"3\"");
    } else if std::env::var("CARGO_FEATURE_EL2").is_ok() {
        println!("cargo::rustc-cfg=el=\"2\"");
    } else {
        println!("cargo::rustc-cfg=el=\"1\"");
    }
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}
