use smallvec::SmallVec;

use std::slice;

use crate::ir::{Type, VecType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Value(SmallVec<[u64; 2]>);

impl Value {
    pub fn from_u8(val: u8) -> Self {
        let mut v = Self::new(1);
        *v.u8_mut() = val;

        v
    }

    pub fn from_u16(val: u16) -> Self {
        let mut v = Self::new(2);
        *v.u16_mut() = val;

        v
    }

    pub fn from_u32(val: u32) -> Self {
        let mut v = Self::new(4);
        *v.u32_mut() = val;

        v
    }

    pub fn from_u64(val: u64) -> Self {
        let mut v = Self::new(8);
        *v.u64_mut() = val;

        v
    }

    pub fn from_i8(val: i8) -> Self {
        let mut v = Self::new(1);
        *v.u8_mut() = val as u8;

        v
    }

    pub fn from_i16(val: i16) -> Self {
        let mut v = Self::new(2);
        *v.u16_mut() = val as u16;

        v
    }

    pub fn from_i32(val: i32) -> Self {
        let mut v = Self::new(4);
        *v.u32_mut() = val as u32;

        v
    }

    pub fn from_i64(val: i64) -> Self {
        let mut v = Self::new(8);
        *v.u64_mut() = val as u64;

        v
    }

    pub fn from_f32(val: f32) -> Self {
        let mut v = Self::new(4);
        *v.u32_mut() = val.to_bits();

        v
    }

    pub fn from_f64(val: f64) -> Self {
        let mut v = Self::new(8);
        *v.u64_mut() = val.to_bits();

        v
    }

    pub fn from_u8x4(val: [u8; 4]) -> Self {
        let mut v = Self::new(4);

        for i in 0..4 {
            v.u8_slice_mut()[i] = val[i];
        }
        v
    }

    pub fn from_u16x2(val: [u16; 2]) -> Self {
        let mut v = Self::new(4);

        for i in 0..2 {
            v.u16_slice_mut()[i] = val[i];
        }
        v
    }

    pub fn from_u32x2(val: [u32; 2]) -> Self {
        let mut v = Self::new(8);

        for i in 0..2 {
            v.u32_slice_mut()[i] = val[i];
        }
        v
    }

    pub fn from_u8x8(val: [u8; 8]) -> Self {
        let mut v = Self::new(8);

        for i in 0..8 {
            v.u8_slice_mut()[i] = val[i];
        }
        v
    }

    pub fn from_u16x4(val: [u16; 4]) -> Self {
        let mut v = Self::new(8);

        for i in 0..4 {
            v.u16_slice_mut()[i] = val[i];
        }
        v
    }

    pub fn from_u32x4(val: [u32; 4]) -> Self {
        let mut v = Self::new(16);

        for i in 0..4 {
            v.u32_slice_mut()[i] = val[i];
        }
        v
    }

    pub fn from_u64x2(val: [u64; 2]) -> Self {
        let mut v = Self::new(16);

        for i in 0..2 {
            v.u64_slice_mut()[i] = val[i];
        }
        v
    }

    pub fn new(len: usize) -> Self {
        let len = usize::max(len, 16);
        let len = if len % 8 == 0 { len / 8 } else { len / 8 + 1 };

        let mut vec = SmallVec::with_capacity(len);
        vec.resize_with(len, || 0);

        Value(vec)
    }

    pub fn truncate_to(mut self, ty: Type) -> Self {
        match ty {
            Type::I64 | Type::U64 | Type::Vec(VecType::U8, 8) => {
                self.0.truncate(1);
            }
            Type::I32 | Type::U32 | Type::Vec(VecType::U8, 4) => {
                self.0.truncate(1);
                *self.u64_mut() = *self.u32_mut() as u64;
            }
            Type::I16 | Type::U16 => {
                self.0.truncate(1);
                *self.u64_mut() = *self.u16_mut() as u64;
            }
            Type::I8 | Type::U8 => {
                self.0.truncate(1);
                *self.u64_mut() = *self.u8_mut() as u64;
            }
            Type::F64 => {
                self.0.truncate(1);
            }
            Type::F32 => {
                self.0.truncate(1);
                *self.u64_mut() = (*self.f32_mut()).to_bits() as u64;
            }
            Type::Void => {
                self.0.truncate(0);
            }
            Type::Bool => {
                self.0.truncate(1);
                *self.u64_mut() = (*self.u8_mut() & 0b1) as u64;
            }
            _ => unimplemented!("truncate_to({:?})", ty),
        }

        self
    }

    pub fn u64_mut(&mut self) -> &mut u64 {
        &mut self.0[0]
    }

    pub fn u32_mut(&mut self) -> &mut u32 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut u32) }
    }

    pub fn u16_mut(&mut self) -> &mut u16 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut u16) }
    }

    pub fn u8_mut(&mut self) -> &mut u8 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut u8) }
    }

    pub fn i64_mut(&mut self) -> &mut i64 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut i64) }
    }

    pub fn i32_mut(&mut self) -> &mut i32 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut i32) }
    }

    pub fn i16_mut(&mut self) -> &mut i16 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut i16) }
    }

    pub fn i8_mut(&mut self) -> &mut i8 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut i8) }
    }

    pub fn f64_mut(&mut self) -> &mut f64 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut f64) }
    }

    pub fn f32_mut(&mut self) -> &mut f32 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut f32) }
    }

    pub fn u64x2_mut(&mut self) -> &mut [u64; 2] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [u64; 2]) }
    }

    pub fn u32x4_mut(&mut self) -> &mut [u32; 4] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [u32; 4]) }
    }

    pub fn u16x8_mut(&mut self) -> &mut [u16; 8] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [u16; 8]) }
    }

    pub fn u8x16_mut(&mut self) -> &mut [u8; 16] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [u8; 16]) }
    }

    pub fn i64x2_mut(&mut self) -> &mut [i64; 2] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [i64; 2]) }
    }

    pub fn i32x4_mut(&mut self) -> &mut [i32; 4] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [i32; 4]) }
    }

    pub fn i16x8_mut(&mut self) -> &mut [i16; 8] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [i16; 8]) }
    }

    pub fn i8x16_mut(&mut self) -> &mut [i8; 16] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [i8; 16]) }
    }

    pub fn f64x2_mut(&mut self) -> &mut [f64; 2] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [f64; 2]) }
    }

    pub fn f32x4_mut(&mut self) -> &mut [f32; 4] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64_mut() as *mut u64 as *mut [f32; 4]) }
    }

    pub fn u64_ref(&self) -> &u64 {
        &self.0[0]
    }

    pub fn u32_ref(&self) -> &u32 {
        unsafe { &*(self.u64_ref() as *const u64 as *const u32) }
    }

    pub fn u16_ref(&self) -> &u16 {
        unsafe { &*(self.u64_ref() as *const u64 as *const u16) }
    }

    pub fn u8_ref(&self) -> &u8 {
        unsafe { &*(self.u64_ref() as *const u64 as *const u8) }
    }

    pub fn i64_ref(&self) -> &i64 {
        unsafe { &*(self.u64_ref() as *const u64 as *const i64) }
    }

    pub fn i32_ref(&self) -> &i32 {
        unsafe { &*(self.u64_ref() as *const u64 as *const i32) }
    }

    pub fn i16_ref(&self) -> &i16 {
        unsafe { &*(self.u64_ref() as *const u64 as *const i16) }
    }

    pub fn i8_ref(&self) -> &i8 {
        unsafe { &*(self.u64_ref() as *const u64 as *const i8) }
    }

    pub fn f64_ref(&self) -> &f64 {
        unsafe { &*(self.u64_ref() as *const u64 as *const f64) }
    }

    pub fn f32_ref(&self) -> &f32 {
        unsafe { &*(self.u64_ref() as *const u64 as *const f32) }
    }

    pub fn u64x2_ref(&self) -> &[u64; 2] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [u64; 2]) }
    }

    pub fn u32x4_ref(&self) -> &[u32; 4] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [u32; 4]) }
    }

    pub fn u16x8_ref(&self) -> &[u16; 8] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [u16; 8]) }
    }

    pub fn u8x16_ref(&self) -> &[u8; 16] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [u8; 16]) }
    }

    pub fn i64x2_ref(&self) -> &[i64; 2] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [i64; 2]) }
    }

    pub fn i32x4_ref(&self) -> &[i32; 4] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [i32; 4]) }
    }

    pub fn i16x8_ref(&self) -> &[i16; 8] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [i16; 8]) }
    }

    pub fn i8x16_ref(&self) -> &[i8; 16] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [i8; 16]) }
    }

    pub fn f64x2_ref(&self) -> &[f64; 2] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [f64; 2]) }
    }

    pub fn f32x4_ref(&self) -> &[f32; 4] {
        unsafe { &*(self.u64_ref() as *const u64 as *const [f32; 4]) }
    }

    pub fn u64(&self) -> u64 {
        *self.u64_ref()
    }

    pub fn u32(&self) -> u32 {
        *self.u32_ref()
    }

    pub fn u16(&self) -> u16 {
        *self.u16_ref()
    }

    pub fn u8(&self) -> u8 {
        *self.u8_ref()
    }

    pub fn i64(&self) -> i64 {
        *self.i64_ref()
    }

    pub fn i32(&self) -> i32 {
        *self.i32_ref()
    }

    pub fn i16(&self) -> i16 {
        *self.i16_ref()
    }

    pub fn i8(&self) -> i8 {
        *self.i8_ref()
    }

    pub fn f64(&self) -> f64 {
        *self.f64_ref()
    }

    pub fn f32(&self) -> f32 {
        *self.f32_ref()
    }

    pub fn u64x2(&self) -> [u64; 2] {
        *self.u64x2_ref()
    }

    pub fn u32x4(&self) -> [u32; 4] {
        *self.u32x4_ref()
    }

    pub fn u16x8(&self) -> [u16; 8] {
        *self.u16x8_ref()
    }

    pub fn u8x16(&self) -> [u8; 16] {
        *self.u8x16_ref()
    }

    pub fn i64x2(&self) -> [i64; 2] {
        *self.i64x2_ref()
    }

    pub fn i32x4(&self) -> [i32; 4] {
        *self.i32x4_ref()
    }

    pub fn i16x8(&self) -> [i16; 8] {
        *self.i16x8_ref()
    }

    pub fn i8x16(&self) -> [i8; 16] {
        *self.i8x16_ref()
    }

    pub fn f64x2(&self) -> [f64; 2] {
        *self.f64x2_ref()
    }

    pub fn f32x4(&self) -> [f32; 4] {
        *self.f32x4_ref()
    }

    pub fn u64_slice_ref(&self) -> &[u64] {
        unsafe { slice::from_raw_parts(self.u64_ref(), self.0.len()) }
    }

    pub fn u32_slice_ref(&self) -> &[u32] {
        unsafe { slice::from_raw_parts(self.u32_ref(), self.0.len() * 2) }
    }

    pub fn u16_slice_ref(&self) -> &[u16] {
        unsafe { slice::from_raw_parts(self.u16_ref(), self.0.len() * 4) }
    }

    pub fn u8_slice_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.u8_ref(), self.0.len() * 8) }
    }

    pub fn i64_slice_ref(&self) -> &[i64] {
        unsafe { slice::from_raw_parts(self.i64_ref(), self.0.len()) }
    }

    pub fn i32_slice_ref(&self) -> &[i32] {
        unsafe { slice::from_raw_parts(self.i32_ref(), self.0.len() * 2) }
    }

    pub fn i16_slice_ref(&self) -> &[i16] {
        unsafe { slice::from_raw_parts(self.i16_ref(), self.0.len() * 4) }
    }

    pub fn i8_slice_ref(&self) -> &[i8] {
        unsafe { slice::from_raw_parts(self.i8_ref(), self.0.len() * 8) }
    }

    pub fn f64_slice_ref(&self) -> &[f64] {
        unsafe { slice::from_raw_parts(self.f64_ref(), self.0.len()) }
    }

    pub fn f32_slice_ref(&self) -> &[f32] {
        unsafe { slice::from_raw_parts(self.f32_ref(), self.0.len() * 2) }
    }

    pub fn u64_slice_mut(&mut self) -> &mut [u64] {
        unsafe { slice::from_raw_parts_mut(self.u64_mut(), self.0.len()) }
    }

    pub fn u32_slice_mut(&mut self) -> &mut [u32] {
        unsafe { slice::from_raw_parts_mut(self.u32_mut(), self.0.len() * 2) }
    }

    pub fn u16_slice_mut(&mut self) -> &mut [u16] {
        unsafe { slice::from_raw_parts_mut(self.u16_mut(), self.0.len() * 4) }
    }

    pub fn u8_slice_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.u8_mut(), self.0.len() * 8) }
    }

    pub fn i64_slice_mut(&mut self) -> &mut [i64] {
        unsafe { slice::from_raw_parts_mut(self.i64_mut(), self.0.len()) }
    }

    pub fn i32_slice_mut(&mut self) -> &mut [i32] {
        unsafe { slice::from_raw_parts_mut(self.i32_mut(), self.0.len() * 2) }
    }

    pub fn i16_slice_mut(&mut self) -> &mut [i16] {
        unsafe { slice::from_raw_parts_mut(self.i16_mut(), self.0.len() * 4) }
    }

    pub fn i8_slice_mut(&mut self) -> &mut [i8] {
        unsafe { slice::from_raw_parts_mut(self.i8_mut(), self.0.len() * 8) }
    }

    pub fn f64_slice_mut(&mut self) -> &mut [f64] {
        unsafe { slice::from_raw_parts_mut(self.f64_mut(), self.0.len()) }
    }

    pub fn f32_slice_mut(&mut self) -> &mut [f32] {
        unsafe { slice::from_raw_parts_mut(self.f32_mut(), self.0.len() * 2) }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::from_u8(v as u8)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::from_u8(v)
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Value::from_u16(v)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::from_u32(v)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::from_u64(v)
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Value::from_i8(v)
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Value::from_i16(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::from_i32(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::from_i64(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::from_f32(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::from_f64(v)
    }
}
