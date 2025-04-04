#![no_std]

mod align;
pub mod ifhal;
pub mod paging;
pub mod region;
pub use align::*;

pub use paging::{PhysAddr, VirtAddr};

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;
