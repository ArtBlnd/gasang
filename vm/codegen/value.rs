use smallvec::SmallVec;

use crate::ir::Type;

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
                *self.u64() = *self.u32() as u64;
            }
            Type::I16 | Type::U16 => {
                self.0.truncate(1);
                *self.u64() = *self.u16() as u64;
            }
            Type::I8 | Type::U8 => {
                self.0.truncate(1);
                *self.u64() = *self.u8() as u64;
            }
            Type::F64 => {
                self.0.truncate(1);
            }
            Type::F32 => {
                self.0.truncate(1);
                *self.u64() = (*self.f32()).to_bits() as u64;
            }
            Type::Void => {
                self.0.truncate(0);
            }
            Type::Bool => {
                self.0.truncate(1);
                *self.u64() = (*self.u8() & 0b1) as u64;
            }
            Type::Vec(_ty, _sz) => {
                todo!()
            }
        }

        self
    }

    pub fn u64(&mut self) -> &mut u64 {
        &mut self.0[0]
    }

    pub fn u32(&mut self) -> &mut u32 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64() as *mut u64 as *mut u32) }
    }

    pub fn u16(&mut self) -> &mut u16 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64() as *mut u64 as *mut u16) }
    }

    pub fn u8(&mut self) -> &mut u8 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64() as *mut u64 as *mut u8) }
    }

    pub fn i64(&mut self) -> &mut i64 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64() as *mut u64 as *mut i64) }
    }

    pub fn i32(&mut self) -> &mut i32 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64() as *mut u64 as *mut i32) }
    }

    pub fn i16(&mut self) -> &mut i16 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64() as *mut u64 as *mut i16) }
    }

    pub fn i8(&mut self) -> &mut i8 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64() as *mut u64 as *mut i8) }
    }

    pub fn f64(&mut self) -> &mut f64 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64() as *mut u64 as *mut f64) }
    }

    pub fn f32(&mut self) -> &mut f32 {
        assert!(!self.0.is_empty());
        unsafe { &mut *(self.u64() as *mut u64 as *mut f32) }
    }

    pub fn u64x2(&mut self) -> &mut [u64; 2] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [u64; 2]) }
    }

    pub fn u32x4(&mut self) -> &mut [u32; 4] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [u32; 4]) }
    }

    pub fn u16x8(&mut self) -> &mut [u16; 8] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [u16; 8]) }
    }

    pub fn u8x16(&mut self) -> &mut [u8; 16] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [u8; 16]) }
    }

    pub fn i64x2(&mut self) -> &mut [i64; 2] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [i64; 2]) }
    }

    pub fn i32x4(&mut self) -> &mut [i32; 4] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [i32; 4]) }
    }

    pub fn i16x8(&mut self) -> &mut [i16; 8] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [i16; 8]) }
    }

    pub fn i8x16(&mut self) -> &mut [i8; 16] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [i8; 16]) }
    }

    pub fn f64x2(&mut self) -> &mut [f64; 2] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [f64; 2]) }
    }

    pub fn f32x4(&mut self) -> &mut [f32; 4] {
        assert!(self.0.len() >= 2);
        unsafe { &mut *(self.u64() as *mut u64 as *mut [f32; 4]) }
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        let mut v = Value::new();
        *v.u64() = value;
        v
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        let mut v = Value::new();
        *v.u32() = value;
        v
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        let mut v = Value::new();
        *v.u16() = value;
        v
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        let mut v = Value::new();
        *v.u8() = value;
        v
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        let mut v = Value::new();
        *v.i64() = value;
        v
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        let mut v = Value::new();
        *v.i32() = value;
        v
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        let mut v = Value::new();
        *v.i16() = value;
        v
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        let mut v = Value::new();
        *v.i8() = value;
        v
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        let mut v = Value::new();
        *v.u64() = value as u64;
        v
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let mut value = Value::new();

        *value.u64() = 0x1234567890abcdef;
        assert_eq!(*value.u64(), 0x1234567890abcdef);
    }

    #[test]
    fn test2() {
        let mut value = Value::new();

        *value.u32() = 0x12345678;
        assert_eq!(*value.u32(), 0x12345678);
    }

    #[test]
    fn test3() {
        let mut value = Value::new();

        *value.u16() = 0x1234;
        assert_eq!(*value.u16(), 0x1234);
    }

    #[test]
    fn test4() {
        let mut value = Value::new();

        *value.u8() = 0x12;
        assert_eq!(*value.u8(), 0x12);
    }

    #[test]
    fn test5() {
        let mut value = Value::new();

        *value.i64() = -0x1234567890abcdef;
        assert_eq!(*value.i64(), -0x1234567890abcdef);
    }

    #[test]
    fn test6() {
        let mut value = Value::new();

        *value.i32() = -0x12345678;
        assert_eq!(*value.i32(), -0x12345678);
    }

    #[test]
    fn test7() {
        let mut value = Value::new();

        *value.i16() = -0x1234;
        assert_eq!(*value.i16(), -0x1234);
    }

    #[test]
    fn test8() {
        let mut value = Value::new();

        *value.i8() = -0x12;
        assert_eq!(*value.i8(), -0x12);
    }

    #[test]
    fn test9() {
        let mut value = Value::new();

        *value.f64() = 0.123456789;
        assert_eq!(*value.f64(), 0.123456789);
    }

    #[test]
    fn test10() {
        let mut value = Value::new();

        *value.f32() = 0.123_456_79;
        assert_eq!(*value.f32(), 0.123_456_79);
    }

    #[test]
    fn test11() {
        let mut value = Value::new();

        *value.u64x2() = [0x1234567890abcdef, 0x1234567890abcdef];
        assert_eq!(*value.u64x2(), [0x1234567890abcdef, 0x1234567890abcdef]);
    }

    #[test]
    fn test12() {
        let mut value = Value::new();

        *value.u32x4() = [0x12345678, 0x12345678, 0x12345678, 0x12345678];
        assert_eq!(
            *value.u32x4(),
            [0x12345678, 0x12345678, 0x12345678, 0x12345678]
        );
    }
}
