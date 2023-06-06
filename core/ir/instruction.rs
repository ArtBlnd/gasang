use crate::Interrupt;

use super::{IrType, IrValue, TypeOf};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IrInst {
    Add {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Sub {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Mul {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Div {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Rem {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Neg {
        dst: IrValue,
        src: IrValue,
    },
    BitAnd {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    BitOr {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    BitXor {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    BitNot {
        dst: IrValue,
        src: IrValue,
    },
    Shl {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Shr {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Assign {
        dst: IrValue,
        src: IrValue,
    },
    Load {
        dst: IrValue,
        src: IrValue,
    },
    Store {
        dst: IrValue,
        src: IrValue,
    },
    ZextCast {
        dst: IrValue,
        src: IrValue,
    },
    SextCast {
        dst: IrValue,
        src: IrValue,
    },
    Interrupt(Interrupt),
    Intrinsic(IrIntrinsic),
}

impl TypeOf for IrInst {
    fn ty(&self) -> IrType {
        match self {
            Self::Add { dst, .. } => dst.ty(),
            Self::Sub { dst, .. } => dst.ty(),
            Self::Mul { dst, .. } => dst.ty(),
            Self::Div { dst, .. } => dst.ty(),
            Self::Rem { dst, .. } => dst.ty(),
            Self::Neg { dst, .. } => dst.ty(),
            Self::BitAnd { dst, .. } => dst.ty(),
            Self::BitOr { dst, .. } => dst.ty(),
            Self::BitXor { dst, .. } => dst.ty(),
            Self::BitNot { dst, .. } => dst.ty(),
            Self::Shl { dst, .. } => dst.ty(),
            Self::Shr { dst, .. } => dst.ty(),
            Self::Assign { dst, .. } => dst.ty(),
            Self::Load { dst, .. } => dst.ty(),
            Self::Store { dst, .. } => dst.ty(),
            Self::ZextCast { dst, .. } => dst.ty(),
            Self::SextCast { dst, .. } => dst.ty(),
            Self::Interrupt(_) => IrType::Void,
            Self::Intrinsic(_) => IrType::Void,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IrIntrinsic {}
