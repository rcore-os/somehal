#[cfg(el = "1")]
mod el1;
#[cfg(el = "2")]
mod el2;

#[cfg(el = "1")]
pub use el1::*;
#[cfg(el = "2")]
pub use el2::*;
