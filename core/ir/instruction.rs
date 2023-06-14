use crate::Interrupt;

use super::{Flag, IrType, IrValue, Reordering, TypeOf};

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
    LogicalAnd {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    LogicalOr {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    LogicalXor {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    LogicalNot {
        dst: IrValue,
        src: IrValue,
    },
    Shl {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    /// Logical shift right
    Lshr {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    /// Arithmetic shift right
    Ashr {
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
    MoveFlag {
        dst: IrValue,
        dst_pos: usize,
        flag: Flag,
    },
    /// A memory fence
    Fence {
        ordering: Reordering,
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
            Self::BitAnd { dst, .. } => dst.ty(),
            Self::BitOr { dst, .. } => dst.ty(),
            Self::BitXor { dst, .. } => dst.ty(),
            Self::BitNot { dst, .. } => dst.ty(),
            Self::LogicalAnd { dst, .. } => dst.ty(),
            Self::LogicalOr { dst, .. } => dst.ty(),
            Self::LogicalXor { dst, .. } => dst.ty(),
            Self::LogicalNot { dst, .. } => dst.ty(),
            Self::Shl { dst, .. } => dst.ty(),
            Self::Lshr { dst, .. } => dst.ty(),
            Self::Ashr { dst, .. } => dst.ty(),
            Self::Assign { dst, .. } => dst.ty(),
            Self::Load { dst, .. } => dst.ty(),
            Self::Store { dst, .. } => dst.ty(),
            Self::ZextCast { dst, .. } => dst.ty(),
            Self::SextCast { dst, .. } => dst.ty(),
            Self::MoveFlag { dst, .. } => dst.ty(),
            Self::Fence { .. } => IrType::Void,
            Self::Interrupt(_) => IrType::Void,
            Self::Intrinsic(_) => IrType::Void,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IrIntrinsic {}
