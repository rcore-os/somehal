#![cfg_attr(not(test), no_std)]
#![feature(pointer_is_aligned_to)]

mod addr;
mod iter;
mod table;
use core::{alloc::Layout, fmt::Debug};

pub use addr::*;
pub use table::{MapConfig, PageTableRef};

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;

#[derive(Debug, Clone, Copy)]
pub struct PTEInfo<P: PTEGeneric> {
    pub level: usize,
    pub vaddr: VirtAddr,
    pub pte: P,
}

pub trait TableGeneric: Sync + Send + Clone + Copy + 'static {
    type PTE: PTEGeneric;

    const PAGE_SIZE: usize = 0x1000;
    const LEVEL: usize = 4;
    const VALID_BITS: usize = 12 + Self::LEVEL * 9;
    // 大页最高支持的级别
    const MAX_BLOCK_LEVEL: usize = 3;
    const TABLE_LEN: usize = Self::PAGE_SIZE / core::mem::size_of::<Self::PTE>();
    fn flush(vaddr: Option<VirtAddr>);
}

pub trait PTEGeneric: Debug + Sync + Send + Clone + Copy + Sized + 'static {
    fn valid(&self) -> bool;
    fn paddr(&self) -> PhysAddr;
    fn set_paddr(&mut self, paddr: PhysAddr);
    fn set_valid(&mut self, valid: bool);
    fn is_huge(&self) -> bool;
    fn set_is_huge(&mut self, b: bool);
}

pub trait Access {
    /// Alloc memory for a page table entry.
    ///
    /// # Safety
    ///
    /// should be deallocated by [`dealloc`].
    unsafe fn alloc(&mut self, layout: Layout) -> Option<PhysAddr>;
    /// dealloc memory for a page table entry.
    ///
    /// # Safety
    ///
    /// ptr must be allocated by [`alloc`].
    unsafe fn dealloc(&mut self, ptr: PhysAddr, layout: Layout);

    fn phys_to_mut(&self, phys: PhysAddr) -> *mut u8;
}

use thiserror::Error;

/// The error type for page table operation failures.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum PagingError {
    #[error("can't allocate memory")]
    NoMemory,
    #[error("{0} is not aligned")]
    NotAligned(&'static str),
    #[error("not mapped")]
    NotMapped,
    #[error("already mapped")]
    AlreadyMapped,
}

/// The specialized `Result` type for page table operations.
pub type PagingResult<T = ()> = Result<T, PagingError>;
