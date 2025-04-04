use core::ptr::NonNull;

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

            #[inline(always)]
            pub const fn new(value: $t) -> Self {
                Self(value)
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
            fn add_assign(&mut self, rhs: $t) {
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

        impl core::ops::Sub<Self> for $name {
            type Output = $t;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                self.0 - rhs.0
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

impl VirtAddr {
    #[inline(always)]
    pub fn as_ptr(self) -> *mut u8 {
        self.0 as _
    }
}

impl From<*mut u8> for VirtAddr {
    #[inline(always)]
    fn from(val: *mut u8) -> Self {
        Self(val as _)
    }
}

impl From<NonNull<u8>> for VirtAddr {
    #[inline(always)]
    fn from(val: NonNull<u8>) -> Self {
        Self(val.as_ptr() as _)
    }
}

impl From<*const u8> for VirtAddr {
    #[inline(always)]
    fn from(val: *const u8) -> Self {
        Self(val as _)
    }
}

#[cfg(target_pointer_width = "64")]
impl From<u64> for PhysAddr {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Self(value as _)
    }
}

#[cfg(target_pointer_width = "32")]
impl From<u32> for PhysAddr {
    #[inline(always)]
    fn from(value: u32) -> Self {
        Self(value as _)
    }
}
