use smallvec::SmallVec;

use std::slice;

use crate::ir::Type;


#[derive(Debug, Clone)]
pub struct Value(SmallVec<[u64; 2]>);

impl Value {
    pub fn new() -> Self {
        Value(SmallVec::from_buf([0; 2]))
    }

    pub fn truncate_to(mut self, ty: Type) -> Self {
        match ty {
            Type::I64 | Type::U64 => {
                self.0.truncate(1);
            }
            Type::I32 | Type::U32 => {
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
            Type::Vec(_ty, _sz) => {
                todo!()
            }
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

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        let mut v = Value::new();
        *v.u64_mut() = value;
        v
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        let mut v = Value::new();
        *v.u32_mut() = value;
        v
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        let mut v = Value::new();
        *v.u16_mut() = value;
        v
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        let mut v = Value::new();
        *v.u8_mut() = value;
        v
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        let mut v = Value::new();
        *v.i64_mut() = value;
        v
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        let mut v = Value::new();
        *v.i32_mut() = value;
        v
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        let mut v = Value::new();
        *v.i16_mut() = value;
        v
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        let mut v = Value::new();
        *v.i8_mut() = value;
        v
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        let mut v = Value::new();
        *v.u64_mut() = value as u64;
        v
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let mut value = Value::new();

        *value.u64_mut() = 0x1234567890abcdef;
        assert_eq!(*value.u64_mut(), 0x1234567890abcdef);
    }

    #[test]
    fn test2() {
        let mut value = Value::new();

        *value.u32_mut() = 0x12345678;
        assert_eq!(*value.u32_mut(), 0x12345678);
    }

    #[test]
    fn test3() {
        let mut value = Value::new();

        *value.u16_mut() = 0x1234;
        assert_eq!(*value.u16_mut(), 0x1234);
    }

    #[test]
    fn test4() {
        let mut value = Value::new();

        *value.u8_mut() = 0x12;
        assert_eq!(*value.u8_mut(), 0x12);
    }

    #[test]
    fn test5() {
        let mut value = Value::new();

        *value.i64_mut() = -0x1234567890abcdef;
        assert_eq!(*value.i64_mut(), -0x1234567890abcdef);
    }

    #[test]
    fn test6() {
        let mut value = Value::new();

        *value.i32_mut() = -0x12345678;
        assert_eq!(*value.i32_mut(), -0x12345678);
    }

    #[test]
    fn test7() {
        let mut value = Value::new();

        *value.i16_mut() = -0x1234;
        assert_eq!(*value.i16_mut(), -0x1234);
    }

    #[test]
    fn test8() {
        let mut value = Value::new();

        *value.i8_mut() = -0x12;
        assert_eq!(*value.i8_mut(), -0x12);
    }

    #[test]
    fn test9() {
        let mut value = Value::new();

        *value.f64_mut() = 0.123456789;
        assert_eq!(*value.f64_mut(), 0.123456789);
    }

    #[test]
    fn test10() {
        let mut value = Value::new();

        *value.f32_mut() = 0.123_456_79;
        assert_eq!(*value.f32_mut(), 0.123_456_79);
    }

    #[test]
    fn test11() {
        let mut value = Value::new();

        *value.u64x2_mut() = [0x1234567890abcdef, 0x1234567890abcdef];
        assert_eq!(*value.u64x2_mut(), [0x1234567890abcdef, 0x1234567890abcdef]);
    }

    #[test]
    fn test12() {
        let mut value = Value::new();

        *value.u32x4_mut() = [0x12345678, 0x12345678, 0x12345678, 0x12345678];
        assert_eq!(
            *value.u32x4_mut(),
            [0x12345678, 0x12345678, 0x12345678, 0x12345678]
        );
    }
}
