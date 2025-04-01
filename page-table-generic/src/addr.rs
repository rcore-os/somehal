macro_rules! def_addr {
    ($name:ident) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(usize);

        impl From<usize> for $name {
            #[inline(always)]
            fn from(value: usize) -> Self {
                Self(value)
            }
        }

        impl $name {
            #[inline(always)]
            pub fn raw(&self) -> usize {
                self.0
            }
        }

        impl core::ops::Add<usize> for $name {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: usize) -> Self::Output {
                Self(self.0 + rhs)
            }
        }

        impl core::ops::AddAssign<usize> for $name {
            #[inline(always)]
            fn add_assign(&mut self, rhs: usize) {
                self.0 += rhs;
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:#x}", self.0)
            }
        }
    };
}

def_addr!(PhysAddr);
def_addr!(VirtAddr);
