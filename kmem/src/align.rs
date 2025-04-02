use crate::{PhysAddr, VirtAddr};

pub trait IntAlign {
    fn align_down(self, align: usize) -> Self;
    fn align_up(self, align: usize) -> Self;
}

#[link_boot::link_boot]
mod _m {
    fn _align_down(x: usize, align: usize) -> usize {
        x & !(align - 1)
    }

    fn _align_up(x: usize, align: usize) -> usize {
        (x + align - 1) & !(align - 1)
    }
}

impl IntAlign for usize {
    #[inline(always)]
    fn align_down(self, align: usize) -> usize {
        _align_down(self, align)
    }

    #[inline(always)]
    fn align_up(self, align: usize) -> usize {
        _align_up(self, align)
    }
}

impl IntAlign for PhysAddr {
    #[inline(always)]
    fn align_down(self, align: usize) -> Self {
        _align_down(self.raw(), align).into()
    }
    #[inline(always)]
    fn align_up(self, align: usize) -> Self {
        _align_up(self.raw(), align).into()
    }
}

impl IntAlign for VirtAddr {
    #[inline(always)]
    fn align_down(self, align: usize) -> Self {
        _align_down(self.raw(), align).into()
    }
    #[inline(always)]
    fn align_up(self, align: usize) -> Self {
        _align_up(self.raw(), align).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align() {
        assert_eq!(0x100.align_up(0x1000), 0x1000);
        assert_eq!(0x100.align_down(0x1000), 0x0);
    }
}
