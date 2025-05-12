use core::{
    cell::UnsafeCell,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::{ArchIf, CpuIdx, arch::Arch, mem::percpu};

use super::{PERCPU_0, percpu_data};

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
        self.data.get() as usize - percpu().as_ptr() as usize
    }

    fn get_remote_ptr(&self, cpu_idx: CpuIdx) -> *mut T {
        unsafe {
            let addr = percpu_data().as_mut().as_ptr() as usize
                + cpu_idx.raw() * PERCPU_0.size
                + self.offset();
            addr as *mut T
        }
    }

    fn get_ptr(&self) -> *mut T {
        let addr = Arch::get_this_percpu_data_ptr() + self.offset();
        addr.raw() as *mut T
    }

    pub fn read_current(&self) -> &T {
        unsafe { &*self.get_ptr() }
    }

    pub fn write_current_raw(&self, val: T) {
        unsafe {
            *self.get_ptr() = val;
        }
    }

    pub fn remote_ref(&self, cpu_idx: usize) -> &T {
        unsafe { &*self.get_remote_ptr(cpu_idx.into()) }
    }
}

impl<T> Deref for PerCpuData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.read_current()
    }
}

impl<T> DerefMut for PerCpuData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.get_ptr() }
    }
}

impl<T: Debug> Debug for PerCpuData<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.read_current())
    }
}
