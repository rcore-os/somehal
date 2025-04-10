#![no_std]

mod align;
pub mod ifhal;
pub mod paging;
pub mod region;
pub use align::*;

pub use paging::{GB, KB, MB, PhysAddr, VirtAddr};
