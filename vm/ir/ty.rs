#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Void,
    Bool,
}

impl Type {
    pub fn is_scalar(&self) -> bool {
        match self {
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::F32
            | Type::F64
            | Type::Bool => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Type::F32 | Type::F64 => true,
            _ => false,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Type::U8 | Type::I8 | Type::Bool => 1,
            Type::U16 | Type::I16 => 2,
            Type::U32 | Type::I32 | Type::F32 => 4,
            Type::U64 | Type::I64 | Type::F64 => 8,
            Type::Void => 0,
        }
    }

    pub fn uscalar_from_size(size: usize) -> Type {
        match size {
            1 => Type::U8,
            2 => Type::U16,
            4 => Type::U32,
            8 => Type::U64,
            _ => unreachable!(),
        }
    }

    pub fn iscalar_from_size(size: usize) -> Type {
        match size {
            1 => Type::I8,
            2 => Type::I16,
            4 => Type::I32,
            8 => Type::I64,
            _ => unreachable!(),
        }
    }
}
