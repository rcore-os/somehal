include!(concat!(env!("OUT_DIR"), "/constant.rs"));

pub const STACK_SIZE: usize = KERNEL_STACK_SIZE;
