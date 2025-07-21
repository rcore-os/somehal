use core::{fmt, ops, slice};

#[derive(Clone, Copy)]
pub struct MemoryRegion {
    /// The physical start address of the region.
    pub start: usize,
    /// The physical end address (exclusive) of the region.
    pub end: usize,
    /// The memory type of the memory region.
    ///
    /// Only [`Usable`][MemoryRegionKind::Usable] regions can be freely used.
    pub kind: MemoryRegionKind,
}

impl fmt::Debug for MemoryRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryRegion")
            .field("start", &(self.start as *const u8))
            .field("end", &(self.end as *const u8))
            .field("kind", &self.kind)
            .finish()
    }
}

/// Represents the different types of memory.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
#[repr(C)]
pub enum MemoryRegionKind {
    /// Physical memory
    Ram,
    /// Reserved memory
    Reserved,
    /// Memory mappings created by the bootloader, including the page table and boot info mappings.
    ///
    /// This memory should _not_ be used by the kernel.
    Bootloader,
    /// An unknown memory region reported by the UEFI firmware.
    ///
    /// Contains the UEFI memory type tag.
    UnknownUefi(u32),
    /// An unknown memory region reported by the BIOS firmware.
    ///
    /// Contains the E820 memory type.
    UnknownBios(u32),
}

/// FFI-safe slice of [`MemoryRegion`] structs, semantically equivalent to
/// `&'static mut [MemoryRegion]`.
///
/// This type implements the [`Deref`][core::ops::Deref] and [`DerefMut`][core::ops::DerefMut]
/// traits, so it can be used like a `&mut [MemoryRegion]` slice. It also implements [`From`]
/// and [`Into`] for easy conversions from and to `&'static mut [MemoryRegion]`.
#[derive(Clone)]
#[repr(C)]
pub struct MemoryRegions {
    pub(crate) ptr: *mut MemoryRegion,
    pub(crate) len: usize,
}

impl ops::Deref for MemoryRegions {
    type Target = [MemoryRegion];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl ops::DerefMut for MemoryRegions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl From<&'static mut [MemoryRegion]> for MemoryRegions {
    fn from(regions: &'static mut [MemoryRegion]) -> Self {
        MemoryRegions {
            ptr: regions.as_mut_ptr(),
            len: regions.len(),
        }
    }
}

impl From<MemoryRegions> for &'static mut [MemoryRegion] {
    fn from(regions: MemoryRegions) -> &'static mut [MemoryRegion] {
        unsafe { slice::from_raw_parts_mut(regions.ptr, regions.len) }
    }
}

impl fmt::Debug for MemoryRegions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MemoryRegions [")?;
        if !self.ptr.is_null() {
            for region in self.iter() {
                writeln!(f, "{region:?}")?;
            }
        }
        writeln!(f, "]")
    }
}

unsafe impl Send for MemoryRegions {}
unsafe impl Sync for MemoryRegions {}
