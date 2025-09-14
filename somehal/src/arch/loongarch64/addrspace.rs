pub const VMLINUX_LOAD_ADDRESS: usize = 0x9000000000200000;

pub const PABITS: usize = 48;

const TO_PHYS_MASK: usize = (1 << PABITS) - 1;

pub const fn to_phys(addr: usize) -> usize {
    addr & TO_PHYS_MASK
}
