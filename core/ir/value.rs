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
    B8(u8),
    B16(u16),
    B32(u32),
    B64(u64),
}

impl IrConstant {
    pub fn new(ty: IrType, value: impl Primitive) -> Self {
        match ty {
            IrType::B8 => IrConstant::B8(value.to_u8().unwrap()),
            IrType::B16 => IrConstant::B16(value.to_u16().unwrap()),
            IrType::B32 => IrConstant::B32(value.to_u32().unwrap()),
            IrType::B64 => IrConstant::B64(value.to_u64().unwrap()),
            _ => unreachable!(),
        }
    }
}

impl TypeOf for IrConstant {
    fn ty(&self) -> IrType {
        match self {
            IrConstant::B8(_) => IrType::B8,
            IrConstant::B16(_) => IrType::B16,
            IrConstant::B32(_) => IrType::B32,
            IrConstant::B64(_) => IrType::B64,
        }
    }
}
