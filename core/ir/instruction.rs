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
    And {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Or {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Xor {
        dst: IrValue,
        lhs: IrValue,
        rhs: IrValue,
    },
    Not {
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
    Rotr {
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
    Fence(Reordering),
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
            Self::And { dst, .. } => dst.ty(),
            Self::Or { dst, .. } => dst.ty(),
            Self::Xor { dst, .. } => dst.ty(),
            Self::Not { dst, .. } => dst.ty(),
            Self::Shl { dst, .. } => dst.ty(),
            Self::Lshr { dst, .. } => dst.ty(),
            Self::Ashr { dst, .. } => dst.ty(),
            Self::Rotr { dst, .. } => dst.ty(),
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
