use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search={}", out_dir().display());

    println!("cargo::rustc-link-arg=-Tlink_test.x");
    // #[cfg(not(target_arch = "loongarch64"))]
    // println!("cargo::rustc-link-arg-tests=-no-pie");
    
    // LoongArch64 EFI stub needs PIE support like Linux kernel efistub
    // #[cfg(target_arch = "loongarch64")]
    {
        // println!("cargo::rustc-link-arg-tests=-pie");
    }
    println!("cargo::rustc-link-arg-tests=-znostart-stop-gc");
    println!("cargo::rustc-link-arg-tests=-Map=target/kernel.map");

    let script = "link_test.ld";

    println!("cargo:rerun-if-changed={script}");
    let ld_content = std::fs::read_to_string(script).unwrap();

    std::fs::write(out_dir().join("link_test.x"), ld_content).expect("link.x write failed");
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}
