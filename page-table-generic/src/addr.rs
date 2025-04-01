macro_rules! def_addr {
    ($name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    };
}

def_addr!(PhysAddr);
def_addr!(VirtAddr);
