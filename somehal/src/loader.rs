macro_rules! loader_bin_slice {
    () => {
        include_bytes!(concat!(env!("OUT_DIR"), "/loader.bin"))
    };
}

const LOADER_BIN_LEN: usize = loader_bin_slice!().len();

const fn loader_bin() -> [u8; LOADER_BIN_LEN] {
    let mut buf = [0u8; LOADER_BIN_LEN];
    let bin = loader_bin_slice!();
    buf.copy_from_slice(bin);
    buf
}

#[unsafe(link_section = ".boot_loader")]
pub static LOADER_BIN: [u8; LOADER_BIN_LEN] = loader_bin();
