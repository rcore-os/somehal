pub trait Align {
    fn align_up(&self, align: usize) -> Self;
    fn align_down(&self, align: usize) -> Self;
}

pub trait AlignTo {
    fn is_aligned_to(&self, align: usize) -> bool;
}

impl AlignTo for usize {
    #[inline(always)]
    fn is_aligned_to(&self, align: usize) -> bool {
        (*self as *const u8).is_aligned_to(align)
    }
}

impl Align for usize {
    #[inline(always)]
    fn align_up(&self, align: usize) -> Self {
        if self.is_aligned_to(align) {
            *self
        } else {
            *self + align - *self % align
        }
    }

    #[inline(always)]
    fn align_down(&self, align: usize) -> Self {
        if self.is_aligned_to(align) {
            *self
        } else {
            *self - *self % align
        }
    }
}

impl Align for *const u8 {
    #[inline(always)]
    fn align_up(&self, align: usize) -> Self {
        (*self as usize).align_up(align) as _
    }

    #[inline(always)]
    fn align_down(&self, align: usize) -> Self {
        (*self as usize).align_down(align) as _
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align() {
        assert_eq!(0x1000.align_up(0x1000), 0x1000);
        assert_eq!(0x1000.align_down(0x1000), 0x1000);
        assert_eq!(0x1001.align_up(0x1000), 0x2000);
        assert_eq!(0x1001.align_down(0x1000), 0x1000);
        assert!(0x1000.is_aligned_to(0x1000));
        assert!(!0x1001.is_aligned_to(0x1000));
    }

    #[test]
    fn test_align2() {
        assert_eq!((0x1000 as *const u8).align_up(0x1000), 0x1000 as *const u8);

        assert_eq!(
            (0x1000 as *const u8).align_down(0x1000),
            0x1000 as *const u8
        );

        assert_eq!((0x1001 as *const u8).align_up(0x1000), 0x2000 as *const u8);

        assert_eq!(
            (0x1001 as *const u8).align_down(0x1000),
            0x1000 as *const u8
        );

        assert!((0x1000 as *const u8).is_aligned_to(0x1000));

        assert!(!(0x1001 as *const u8).is_aligned_to(0x1000));
    }
}
