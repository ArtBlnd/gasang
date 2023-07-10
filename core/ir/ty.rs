#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IrType {
    B8,
    B16,
    B32,
    B64,
    B128,
    F32,
    F64,
    Bool,
    Void,
    Vector(VecTy, u32),
}

impl IrType {
    pub fn size_of(self) -> usize {
        match self {
            IrType::B8 => 1,
            IrType::B16 => 2,
            IrType::B32 | IrType::F32 => 4,
            IrType::B64 | IrType::F64 => 8,
            IrType::B128 => 16,
            IrType::Bool => 1,
            IrType::Void => 0,
            IrType::Vector(ty, size) => ty.size_of() * size as usize,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VecTy {
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
}

impl VecTy {
    pub fn size_of(self) -> usize {
        match self {
            VecTy::U8 => 1,
            VecTy::U16 => 2,
            VecTy::U32 | VecTy::F32 => 4,
            VecTy::U64 | VecTy::F64 => 8,
            VecTy::U128 => 16,
        }
    }
}

pub trait TypeOf {
    fn ty(&self) -> IrType;
}
