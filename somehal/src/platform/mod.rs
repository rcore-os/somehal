cfg_if::cfg_if! {
    if #[cfg(use_acpi)]{
        mod acpi;
        pub use acpi::*;
    }else if #[cfg(use_fdt)]{
        mod fdt;
        pub use fdt::*;
    }
}

macro_rules! def_id {
    ($id:ident, $t:ty) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $id($t);

        impl From<$t> for $id {
            fn from(value: $t) -> Self {
                $id(value)
            }
        }

        impl From<$id> for $t {
            fn from(value: $id) -> Self {
                value.0
            }
        }

        impl $id {
            pub const fn new(value: $t) -> Self {
                $id(value)
            }

            pub fn raw(&self) -> $t {
                self.0
            }
        }

        impl core::fmt::Debug for $id {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:#X}", self.0)
            }
        }
    };
}

def_id!(CpuIdx, usize);
def_id!(CpuId, usize);

impl CpuIdx {
    pub fn is_primary(&self) -> bool {
        self.0 == 0
    }
}
