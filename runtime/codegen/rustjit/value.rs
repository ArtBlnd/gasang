use core::ir::{IrConstant, IrType};

use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};

const VALUE_SIZE: usize = 16;

#[derive(Clone, Copy, Debug)]
pub struct RustjitValue {
    raw: [u8; VALUE_SIZE],
    ty: IrType,
}

impl RustjitValue {
    pub fn as_bytes(&self) -> &[u8] {
        &self.raw[..self.ty.size_in_bytes()]
    }

    pub fn from_bytes(bytes: &[u8], ty: IrType) -> Self {
        let mut raw = [0; VALUE_SIZE];
        raw[..ty.size_in_bytes()].copy_from_slice(bytes[..ty.size_in_bytes()].try_into().unwrap());
        Self { raw, ty }
    }
}

impl From<IrConstant> for RustjitValue {
    fn from(value: IrConstant) -> Self {
        let mut raw = [0; VALUE_SIZE];
        let ty = match value {
            IrConstant::U8(value) => {
                raw[0] = value;
                IrType::U8
            }
            IrConstant::U16(value) => {
                raw[..2].copy_from_slice(&value.to_le_bytes());
                IrType::U16
            }
            IrConstant::U32(value) => {
                raw[..4].copy_from_slice(&value.to_le_bytes());
                IrType::U32
            }
            IrConstant::U64(value) => {
                raw[..8].copy_from_slice(&value.to_le_bytes());
                IrType::U64
            }
            IrConstant::I8(value) => {
                raw[0] = value as u8;
                IrType::I8
            }
            IrConstant::I16(value) => {
                raw[..2].copy_from_slice(&value.to_le_bytes());
                IrType::I16
            }
            IrConstant::I32(value) => {
                raw[..4].copy_from_slice(&value.to_le_bytes());
                IrType::I32
            }
            IrConstant::I64(value) => {
                raw[..8].copy_from_slice(&value.to_le_bytes());
                IrType::I64
            }
        };

        RustjitValue { raw, ty }
    }
}

impl From<u64> for RustjitValue {
    fn from(value: u64) -> Self {
        let mut raw = [0; VALUE_SIZE];
        raw.copy_from_slice(&value.to_le_bytes()[..8]);

        Self {
            raw,
            ty: IrType::U64,
        }
    }
}

impl Neg for RustjitValue {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        match self.ty {
            IrType::Bool => {
                let value = self.raw[0] != 0;
                self.raw[0] = !value as u8;
                self
            }
            other => {
                let high = self.raw[other.size_in_bytes()];
                self.raw[other.size_in_bytes()] = high ^ 0x80;
                self
            }
        }
    }
}

impl Not for RustjitValue {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        match self.ty {
            IrType::Bool => {
                let value = self.raw[0] != 0;
                self.raw[0] = !value as u8;
                self
            }
            other => {
                self.raw[..other.size_in_bytes()]
                    .as_mut()
                    .iter_mut()
                    .for_each(|b| *b = !*b);
                self
            }
        }
    }
}

macro_rules! impl_traits {
    ($(
        [$trait_name:ident, $trait_fn:ident, $proc:ident $(, $target_ty:ty)?]
    )*) => {
        $(
            impl $trait_name for RustjitValue {
                type Output = Self;

                fn $trait_fn(mut self, rhs: Self) -> Self::Output {
                    match (self.ty, rhs.ty) {
                        (IrType::I8 | IrType::I16 | IrType::I32 | IrType::I64, IrType::I8 | IrType::I16 | IrType::I32 | IrType::I64) => {
                            let lhs = i64::from_le_bytes([
                                self.raw[0], self.raw[1], self.raw[2], self.raw[3],
                                self.raw[4], self.raw[5], self.raw[6], self.raw[7],
                            ]);
                            let rhs = i64::from_le_bytes([
                                rhs.raw[0], rhs.raw[1], rhs.raw[2], rhs.raw[3],
                                rhs.raw[4], rhs.raw[5], rhs.raw[6], rhs.raw[7],
                            ]);
                            let result = lhs.$proc(rhs $( as $target_ty)?);
                            self.raw[..8].copy_from_slice(&result.to_le_bytes());
                        }
                        (IrType::I128, IrType::I8 | IrType::I16 | IrType::I32 | IrType::I64 | IrType::I128) => {
                            let lhs = i128::from_le_bytes([
                                self.raw[0], self.raw[1], self.raw[2], self.raw[3],
                                self.raw[4], self.raw[5], self.raw[6], self.raw[7],
                                self.raw[8], self.raw[9], self.raw[10], self.raw[11],
                                self.raw[12], self.raw[13], self.raw[14], self.raw[15],
                            ]);
                            let rhs = i128::from_le_bytes([
                                rhs.raw[0], rhs.raw[1], rhs.raw[2], rhs.raw[3],
                                rhs.raw[4], rhs.raw[5], rhs.raw[6], rhs.raw[7],
                                rhs.raw[8], rhs.raw[9], rhs.raw[10], rhs.raw[11],
                                rhs.raw[12], rhs.raw[13], rhs.raw[14], rhs.raw[15],
                            ]);
                            let result = lhs.$proc(rhs $( as $target_ty)?);
                            self.raw[..16].copy_from_slice(&result.to_le_bytes());
                        }
                        (IrType::U8 | IrType::U16 | IrType::U32 | IrType::U64, IrType::U8 | IrType::U16 | IrType::U32 | IrType::U64) => {
                            let lhs = u64::from_le_bytes([
                                self.raw[0], self.raw[1], self.raw[2], self.raw[3],
                                self.raw[4], self.raw[5], self.raw[6], self.raw[7],
                            ]);
                            let rhs = u64::from_le_bytes([
                                rhs.raw[0], rhs.raw[1], rhs.raw[2], rhs.raw[3],
                                rhs.raw[4], rhs.raw[5], rhs.raw[6], rhs.raw[7],
                            ]);
                            let result = lhs.$proc(rhs $( as $target_ty)?);
                            self.raw[..8].copy_from_slice(&result.to_le_bytes());
                        }
                        (IrType::U128, IrType::U8 | IrType::U16 | IrType::U32 | IrType::U64 | IrType::U128) => {
                            let lhs = u128::from_le_bytes([
                                self.raw[0], self.raw[1], self.raw[2], self.raw[3],
                                self.raw[4], self.raw[5], self.raw[6], self.raw[7],
                                self.raw[8], self.raw[9], self.raw[10], self.raw[11],
                                self.raw[12], self.raw[13], self.raw[14], self.raw[15],
                            ]);
                            let rhs = u128::from_le_bytes([
                                rhs.raw[0], rhs.raw[1], rhs.raw[2], rhs.raw[3],
                                rhs.raw[4], rhs.raw[5], rhs.raw[6], rhs.raw[7],
                                rhs.raw[8], rhs.raw[9], rhs.raw[10], rhs.raw[11],
                                rhs.raw[12], rhs.raw[13], rhs.raw[14], rhs.raw[15],
                            ]);
                            let result = lhs.$proc(rhs $( as $target_ty)?);
                            self.raw[..16].copy_from_slice(&result.to_le_bytes());
                        }
                        _ => panic!("Unsupported type for {}: {:?} and {:?}", stringify!($trait_name), self.ty, rhs.ty),
                    }

                    self
                }
            }
        )*
    };
}

impl_traits! {
    [Add, add, wrapping_add]
    [Sub, sub, wrapping_sub]
    [Mul, mul, wrapping_mul]
    [Div, div, wrapping_div]
    [Rem, rem, wrapping_rem]
    [BitAnd, bitand, bitand]
    [BitOr, bitor, bitor]
    [BitXor, bitxor, bitxor]
    [Shl, shl, wrapping_shl, u32]
    [Shr, shr, wrapping_shr, u32]
}
