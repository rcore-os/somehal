use core::{cell::UnsafeCell, fmt::Debug, ops::Deref};

use crate::{get_cpu_local_ptr, percpu_base, percpu_link_start, percpu_size};

#[repr(transparent)]
pub struct PerCpuData<T> {
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for PerCpuData<T> {}
unsafe impl<T> Send for PerCpuData<T> {}

impl<T> PerCpuData<T> {
    pub const fn new(data: T) -> PerCpuData<T> {
        PerCpuData {
            data: UnsafeCell::new(data),
        }
    }

    fn offset(&self) -> usize {
        self.data.get() as usize - percpu_base()
    }

    fn get_remote_ptr(&self, cpu_idx: usize) -> *mut T {
        let addr = percpu_link_start() + cpu_idx * percpu_size() + self.offset();
        addr as *mut T
    }

    fn get_ptr(&self) -> *mut T {
        let addr = get_cpu_local_ptr() as usize + self.offset();
        addr as *mut T
    }

    pub fn read_current(&self) -> &T {
        unsafe { &*self.get_ptr() }
    }

    pub fn write_current(&self, val: T) {
        unsafe {
            *self.get_ptr() = val;
        }
    }

    pub fn remote_ref(&self, cpu_idx: usize) -> &T {
        unsafe { &*self.get_remote_ptr(cpu_idx) }
    }
}

impl<T> Deref for PerCpuData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.read_current()
    }
}

impl<T: Debug> Debug for PerCpuData<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.read_current())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
