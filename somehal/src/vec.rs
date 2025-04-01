#![allow(unused)]

#[link_boot::link_boot]
mod _m {

    pub enum Error {
        Full,
    }

    #[repr(C)]
    pub struct ArrayVec<T, const CAP: usize> {
        // the `len` first elements of the array are initialized
        xs: [Option<T>; CAP],
    }

    impl<T, const CAP: usize> ArrayVec<T, CAP> {
        pub fn new() -> ArrayVec<T, CAP> {
            ArrayVec {
                xs: [const { None }; CAP],
            }
        }

        pub fn len(&self) -> usize {
            self.xs.iter().filter(|x| x.is_some()).count()
        }

        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        pub fn try_push(&mut self, value: T) -> Result<(), Error> {
            if self.len() < CAP {
                self.xs[self.len()] = Some(value);
                Ok(())
            } else {
                Err(Error::Full)
            }
        }
    }

    impl<T, const CAP: usize> Iterator for ArrayVec<T, CAP> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            let mut out = None;
            for i in 0..CAP {
                if let Some(x) = self.xs[i].take() {
                    out = Some(x);
                    break;
                }
            }
            out
        }
    }
}
