macro_rules! def_addr {
    ($name:ident, $t:ty) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($t);

        impl From<$t> for $name {
            #[inline(always)]
            fn from(value: $t) -> Self {
                Self(value)
            }
        }

        impl $name {
            #[inline(always)]
            pub fn raw(&self) -> $t {
                self.0
            }
        }

        impl core::ops::Add<$t> for $name {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: $t) -> Self::Output {
                Self(self.0 + rhs)
            }
        }

        impl core::ops::AddAssign<$t> for $name {
            #[inline(always)]
            fn add_assign(&mut self, rhs:$t) {
                self.0 += rhs;
            }
        }

        impl core::ops::Sub<$t> for $name {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: $t) -> Self::Output {
                Self(self.0 - rhs)
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "0x{:0>16x}", self.0)
            }
        }
    };
}

def_addr!(PhysAddr, usize);
def_addr!(VirtAddr, usize);
