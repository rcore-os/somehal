#![cfg_attr(not(test), no_std)]

pub trait NumAlign {
    fn align_up(self, align: usize) -> Self;
    fn align_down(self, align: usize) -> Self;
}
pub trait NumAssertAlign: Copy {
    fn is_aligned_to(self, align: usize) -> bool;
}

macro_rules! impl_num_align {
    ($t:ty) => {
        impl NumAlign for $t {
            #[inline(always)]
            fn align_up(self, align: usize) -> Self {
                if self.is_aligned_to(align) {
                    self
                } else {
                    (self as usize + align - self as usize % align) as _
                }
            }

            #[inline(always)]
            fn align_down(self, align: usize) -> Self {
                if self.is_aligned_to(align) {
                    self
                } else {
                    (self as usize - self as usize % align) as _
                }
            }
        }
        impl NumAssertAlign for $t {
            #[inline(always)]
            fn is_aligned_to(self, align: usize) -> bool {
                self as usize % align == 0
            }
        }
    };
}

impl_num_align!(usize);
impl_num_align!(u64);
impl_num_align!(u32);

impl<T> NumAlign for *const T {
    #[inline(always)]
    fn align_up(self, align: usize) -> Self {
        (self as usize).align_up(align) as _
    }

    #[inline(always)]
    fn align_down(self, align: usize) -> Self {
        (self as usize).align_down(align) as _
    }
}

impl<T> NumAlign for *mut T {
    #[inline(always)]
    fn align_up(self, align: usize) -> Self {
        (self as usize).align_up(align) as _
    }

    #[inline(always)]
    fn align_down(self, align: usize) -> Self {
        (self as usize).align_down(align) as _
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align() {
        assert_eq!(0x1000usize.align_up(0x1000), 0x1000);
        assert_eq!(0x1000usize.align_down(0x1000), 0x1000);
        assert_eq!(0x1001usize.align_up(0x1000), 0x2000);
        assert_eq!(0x1001usize.align_down(0x1000), 0x1000);
        assert!(0x1000usize.is_aligned_to(0x1000));
        assert!(!0x1001usize.is_aligned_to(0x1000));
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
    }
}
