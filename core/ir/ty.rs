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

pub trait TypeOf {
    fn ty(&self) -> IrType;
}
