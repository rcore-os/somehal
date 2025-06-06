use std::{io::Write, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=link.ld");
    println!("cargo:rustc-link-search={}", out_dir().display());

    let name = "somert.x";
    let mut content = include_str!("link.ld").to_string();

    let mut include_str = String::new();

    if std::env::var("CARGO_FEATURES_KERNEL_ADDR").is_ok() {
        include_str += "
        INCLUDE memory.x
        ";
    }

    content = content.replace("${INCLUDE}", &include_str);

    let mut file = std::fs::File::create(out_dir().join(name))
        .unwrap_or_else(|_| panic!("{name} create failed"));
    file.write_all(content.as_bytes())
        .unwrap_or_else(|_| panic!("{name} write failed"));
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}
