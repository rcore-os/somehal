fn main() {
    println!("cargo::rustc-link-arg-tests=-Tlink.x");
    println!("cargo::rustc-link-arg-tests=-no-pie");
    println!("cargo::rustc-link-arg-tests=-znostart-stop-gc");
}
