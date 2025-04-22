#![allow(unused)]

use core::ops::Index;

pub enum Error {
    Full,
}

#[repr(C)]
pub struct ArrayVec<T, const CAP: usize> {
    // the `len` first elements of the array are initialized
    xs: [Option<T>; CAP],
}

impl<T, const CAP: usize> ArrayVec<T, CAP> {
    pub const fn new() -> ArrayVec<T, CAP> {
        ArrayVec {
            xs: [const { None }; CAP],
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.xs.iter().filter(|x| x.is_some()).count()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn try_push(&mut self, value: T) -> Result<(), Error> {
        if self.len() < CAP {
            self.xs[self.len()] = Some(value);
            Ok(())
        } else {
            Err(Error::Full)
        }
    }
}

impl<T, const CAP: usize> Default for ArrayVec<T, CAP> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP: usize> Index<usize> for ArrayVec<T, CAP> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: usize) -> &Self::Output {
        self.xs[idx].as_ref().unwrap()
    }
}

impl<T: Clone, const CAP: usize> Clone for ArrayVec<T, CAP> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            xs: self.xs.clone(),
        }
    }
}

impl<T, const CAP: usize> Iterator for ArrayVec<T, CAP> {
    type Item = T;

    #[inline(always)]
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
