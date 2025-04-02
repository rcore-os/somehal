macro_rules! def_id {
    ($id:ident, $t:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    };
}

def_id!(CpuHardId, usize);
def_id!(CpuId, usize);
