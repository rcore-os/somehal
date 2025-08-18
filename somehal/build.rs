use kdef_pgtable::*;
use std::{io::Write, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rerun-if-changed=link_base.ld");
    println!("cargo:rustc-link-search={}", out_dir().display());

    let target = std::env::var("TARGET").unwrap();

    println!("cargo::rustc-check-cfg=cfg(hard_float)");
    if target.as_str() == "aarch64-unknown-none" {
        println!("cargo::rustc-cfg=hard_float");
    }

    let kimage_vaddr = KIMAGE_VADDR;
    let page_size = PAGE_SIZE;

    let mut ld = include_str!("link_base.ld").to_string();

    macro_rules! set_var {
        ($v:ident) => {
            ld = ld.replace(concat!("{", stringify!($v), "}"), &format!("{:#x}", $v));
        };
    }

    set_var!(kimage_vaddr);
    set_var!(page_size);

    let ld_name_out = "pie_boot.x";

    let mut file = std::fs::File::create(out_dir().join(ld_name_out))
        .unwrap_or_else(|_| panic!("{ld_name_out} create failed"));
    file.write_all(ld.as_bytes())
        .unwrap_or_else(|_| panic!("{ld_name_out} write failed"));

    let ld = include_str!("link.ld").to_string();
    let ld_name_out = "somehal.x";
    let mut file = std::fs::File::create(out_dir().join(ld_name_out))
        .unwrap_or_else(|_| panic!("{ld_name_out} create failed"));
    file.write_all(ld.as_bytes())
        .unwrap_or_else(|_| panic!("{ld_name_out} write failed"));

    if std::env::var("TARGET").unwrap().contains("aarch64-") {
        aarch64_set_loader();
    }
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}

fn aarch64_set_loader() {
    let builder = bindeps_simple::Builder::new("pie-boot-loader-aarch64")
        .target("aarch64-unknown-none-softfloat")
        .env("RUSTFLAGS", "-C relocation-model=pic -Clink-args=-pie")
        .cargo_args(&["-Z", "build-std=core,alloc"]);

    let output = builder.build().unwrap();

    let loader_path = output.elf;
    let loader_dst = out_dir().join("loader.bin");

    let _ = std::fs::remove_file(&loader_dst);

    let status = Command::new("rust-objcopy")
        .args(["--strip-all", "-O", "binary"])
        .arg(&loader_path)
        .arg(loader_dst)
        .status()
        .expect("objcopy failed");

    assert!(status.success());

    println!("target dir: {}", target_dir().display());

    let _ = std::fs::remove_file(target_dir().join("loader.elf"));
    std::fs::copy(&loader_path, target_dir().join("loader.elf")).unwrap();
}

fn target_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .ancestors()
        .nth(3)
        .unwrap()
        .into()
}
