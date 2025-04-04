use crate::{ArchIf, arch, println};

pub fn rust_main() {
    #[cfg(not(feature = "early-debug"))]
    arch::Arch::init_debugcon();
    println!("Hello, world!");
}
