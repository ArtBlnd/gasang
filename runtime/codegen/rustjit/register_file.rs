use core::{RawRegisterId, RegisterFileDesc};
use std::cell::UnsafeCell;

use crate::codegen::ValueView;

/// Represents a cpu register file.
pub struct RegisterFile {
    desc: RegisterFileDesc,
    file: UnsafeCell<Box<[u8]>>,
}

impl RegisterFile {
    pub fn new(desc: &RegisterFileDesc) -> Self {
        let mut file = Vec::new();
        file.resize(desc.total_size(), 0);
        Self {
            desc: desc.clone(),
            file: UnsafeCell::new(file.into_boxed_slice()),
        }
    }

    /// Get reference of the the register as T
    ///
    /// This function will panic if the size of T and the register size does not match.
    pub fn get<T>(&self, reg: RawRegisterId) -> T
    where
        T: ValueView,
    {
        unsafe {
            let file = &mut *self.file.get();
            let offset = self.desc.register(reg).offset;
            let ptr = file.as_mut().as_mut_ptr().add(offset);
            *(ptr as *const T)
        }
    }

    /// Get mutable reference of the the register as T
    ///
    /// This function will panic if the size of T and the register size does not match.
    pub fn get_mut<T>(&self, reg: RawRegisterId) -> &mut T
    where
        T: ValueView,
    {
        unsafe {
            let file = &mut *self.file.get();
            let offset = self.desc.register(reg).offset;
            let ptr = file.as_mut().as_mut_ptr().add(offset);
            &mut *(ptr as *mut T)
        }
    }
}
