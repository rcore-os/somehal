fn main() {
    println!("cargo::rustc-check-cfg=cfg(fdt)");

    if std::env::var("CARGO_FEA")


    if std::env::var("TARGET").unwrap() == "aarch64-unknown-none" {
        println!("cargo::rustc-cfg=hard_float");
    }
}
