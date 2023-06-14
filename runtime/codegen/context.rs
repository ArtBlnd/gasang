use core::ir::{Flag, IrValue};
use std::mem::size_of;

pub trait Context {
    /// Returns the value of the given IrValue.
    ///
    /// This function panics if the given type T and the type of the IrValue do not match.
    fn get<T: ValueView>(&self, value: IrValue) -> T;

    /// Set the value of the given IrValue.
    ///
    /// This function panics if the given type T and the type of the IrValue do not match
    /// or if the given IrValue is constant.
    fn set<T: ValueView>(&self, value: IrValue, new_value: T);

    fn get_flag(&self, flag: Flag) -> bool;
    fn set_flag(&self, flag: Flag, value: bool);
}

pub trait ValueView: Sized + Copy {
    type Bytes: AsRef<[u8]>;

    fn into_bytes(self) -> Self::Bytes;
    fn from_bytes(bytes: &[u8]) -> Self;
}

macro_rules! impl_value_view_primitive {
    ($ty:ty, $len:literal) => {
        impl ValueView for $ty {
            type Bytes = [u8; $len];

            #[inline(always)]
            fn into_bytes(self) -> Self::Bytes {
                self.to_ne_bytes()
            }

            #[inline(always)]
            fn from_bytes(bytes: &[u8]) -> Self {
                let mut data = [0; $len];
                data.copy_from_slice(bytes);
                <$ty>::from_ne_bytes(data)
            }
        }
    };
}

impl_value_view_primitive!(u8, 1);
impl_value_view_primitive!(u16, 2);
impl_value_view_primitive!(u32, 4);
impl_value_view_primitive!(u64, 8);
impl_value_view_primitive!(u128, 16);
impl_value_view_primitive!(i8, 1);
impl_value_view_primitive!(i16, 2);
impl_value_view_primitive!(i32, 4);
impl_value_view_primitive!(i64, 8);
impl_value_view_primitive!(i128, 16);
impl_value_view_primitive!(f32, 4);
impl_value_view_primitive!(f64, 8);

macro_rules! impl_value_view_composite {
    ($ty:ty) => {
        impl ValueView for $ty {
            type Bytes = [u8; size_of::<Self>()];

            #[inline(always)]
            fn into_bytes(self) -> Self::Bytes {
                debug_assert!(size_of::<Self>() == size_of::<Self::Bytes>());
                unsafe { std::mem::transmute(self) }
            }

            #[inline(always)]
            fn from_bytes(bytes: &[u8]) -> Self {
                debug_assert!(size_of::<Self>() == size_of::<Self::Bytes>());
                let mut data = [0; size_of::<Self>()];
                data.copy_from_slice(bytes);
                unsafe { std::mem::transmute(data) }
            }
        }
    };
}

impl_value_view_composite!([u8; 2]);
impl_value_view_composite!([u8; 4]);
impl_value_view_composite!([u8; 8]);
impl_value_view_composite!([u8; 16]);
impl_value_view_composite!([u8; 32]);
impl_value_view_composite!([u8; 64]);
impl_value_view_composite!([u16; 2]);
impl_value_view_composite!([u16; 4]);
impl_value_view_composite!([u16; 8]);
impl_value_view_composite!([u16; 16]);
impl_value_view_composite!([u32; 2]);
impl_value_view_composite!([u32; 4]);
impl_value_view_composite!([u32; 8]);
impl_value_view_composite!([u64; 2]);
impl_value_view_composite!([u64; 4]);
impl_value_view_composite!([u64; 8]);
impl_value_view_composite!([u128; 1]);
impl_value_view_composite!([u128; 2]);
impl_value_view_composite!([u128; 4]);
