#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IrType {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Void,
    Vector(VecTy, u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VecTy {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
}

impl IrType {
    pub fn size_of(self) -> usize {
        match self {
            IrType::I8 | IrType::U8 => 1,
            IrType::I16 | IrType::U16 => 2,
            IrType::I32 | IrType::U32 | IrType::F32 => 4,
            IrType::I64 | IrType::U64 | IrType::F64 => 8,
            IrType::I128 | IrType::U128 => 16,
            IrType::Bool => 1,
            IrType::Void => 0,
            IrType::Vector(_, size) => todo!(),
        }
    }
}

pub trait TypeOf {
    fn ty(&self) -> IrType;
}
