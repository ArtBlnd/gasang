use crate::{Primitive, RawRegisterId};

use super::{IrType, TypeOf};

/// An representation of a value in the IR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IrValue {
    Variable(IrType, usize),
    Register(IrType, RawRegisterId),
    Constant(IrConstant),
}

impl TypeOf for IrValue {
    fn ty(&self) -> IrType {
        match self {
            IrValue::Register(ty, _) => *ty,
            IrValue::Variable(ty, _) => *ty,
            IrValue::Constant(constant) => constant.ty(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IrConstant {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

impl IrConstant {
    pub fn new(ty: IrType, value: impl Primitive) -> Self {
        match ty {
            IrType::U8 => IrConstant::U8(value.to_u8().unwrap()),
            IrType::U16 => IrConstant::U16(value.to_u16().unwrap()),
            IrType::U32 => IrConstant::U32(value.to_u32().unwrap()),
            IrType::U64 => IrConstant::U64(value.to_u64().unwrap()),
            IrType::I8 => IrConstant::I8(value.to_i8().unwrap()),
            IrType::I16 => IrConstant::I16(value.to_i16().unwrap()),
            IrType::I32 => IrConstant::I32(value.to_i32().unwrap()),
            IrType::I64 => IrConstant::I64(value.to_i64().unwrap()),
            _ => unreachable!(),
        }
    }
}

impl TypeOf for IrConstant {
    fn ty(&self) -> IrType {
        match self {
            IrConstant::U8(_) => IrType::U8,
            IrConstant::U16(_) => IrType::U16,
            IrConstant::U32(_) => IrType::U32,
            IrConstant::U64(_) => IrType::U64,
            IrConstant::I8(_) => IrType::I8,
            IrConstant::I16(_) => IrType::I16,
            IrConstant::I32(_) => IrType::I32,
            IrConstant::I64(_) => IrType::I64,
        }
    }
}
