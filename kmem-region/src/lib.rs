#![no_std]

mod align;
pub mod region;
pub use align::*;
mod addr;
pub mod alloc;

pub use addr::*;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;
