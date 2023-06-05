use core::{RawRegisterId, RegisterFileDesc};

/// Represents a cpu register file.
pub struct RegisterFile {
    desc: RegisterFileDesc,
    file: Box<[u8]>,
}

pub trait RegisterProjection: Sized {}
impl RegisterProjection for u8 {}
impl RegisterProjection for u16 {}
impl RegisterProjection for u32 {}
impl RegisterProjection for u64 {}
impl RegisterProjection for u128 {}
impl RegisterProjection for i8 {}
impl RegisterProjection for i16 {}
impl RegisterProjection for i32 {}
impl RegisterProjection for i64 {}
impl RegisterProjection for i128 {}
impl<const LEN: usize> RegisterProjection for [u8; LEN] {}
impl<const LEN: usize> RegisterProjection for [u16; LEN] {}
impl<const LEN: usize> RegisterProjection for [u32; LEN] {}
impl<const LEN: usize> RegisterProjection for [u64; LEN] {}
impl<const LEN: usize> RegisterProjection for [u128; LEN] {}
impl<const LEN: usize> RegisterProjection for [i8; LEN] {}
impl<const LEN: usize> RegisterProjection for [i16; LEN] {}
impl<const LEN: usize> RegisterProjection for [i32; LEN] {}
impl<const LEN: usize> RegisterProjection for [i64; LEN] {}
impl<const LEN: usize> RegisterProjection for [i128; LEN] {}
impl<const LEN: usize> RegisterProjection for [f32; LEN] {}
impl<const LEN: usize> RegisterProjection for [f64; LEN] {}

impl RegisterFile {
    pub fn new(desc: &RegisterFileDesc) -> Self {
        let mut file = Vec::new();
        file.resize(desc.total_size(), 0);
        Self {
            desc: desc.clone(),
            file: file.into_boxed_slice(),
        }
    }

    /// Get reference of the the register as T
    ///
    /// This function will panic if the size of T and the register size does not match.
    pub fn get_ref<T>(&self, reg: RawRegisterId) -> &T
    where
        T: RegisterProjection,
    {
        unsafe {
            let reg = self.desc.register(reg);
            let ptr = self.file.as_ptr().add(reg.offset);

            assert!(reg.offset + std::mem::size_of::<T>() <= self.file.len());
            assert!(std::mem::size_of::<T>() == reg.size);
            &*(ptr as *const T)
        }
    }

    /// Get mutable reference of the the register as T
    ///
    /// This function will panic if the size of T and the register size does not match.
    pub fn get_mut<T>(&mut self, reg: RawRegisterId) -> &mut T
    where
        T: RegisterProjection,
    {
        unsafe {
            let reg = self.desc.register(reg);
            let ptr = self.file.as_mut_ptr().add(reg.offset);

            assert!(reg.offset + std::mem::size_of::<T>() <= self.file.len());
            assert!(std::mem::size_of::<T>() == reg.size);
            &mut *(ptr as *mut T)
        }
    }
}
