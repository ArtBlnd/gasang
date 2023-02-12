#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VecType {
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
}

impl VecType {
    pub fn size(&self) -> usize {
        match self {
            VecType::U8 | VecType::I8 => 1,
            VecType::U16 | VecType::I16 => 2,
            VecType::U32 | VecType::I32 | VecType::F32 => 4,
            VecType::U64 | VecType::I64 | VecType::F64 => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Vec(VecType, usize),
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
    pub fn u64x2() -> Type {
        Type::Vec(VecType::U64, 2)
    }

    pub fn u32x4() -> Type {
        Type::Vec(VecType::U32, 4)
    }

    pub fn u16x8() -> Type {
        Type::Vec(VecType::U16, 8)
    }

    pub fn u8x16() -> Type {
        Type::Vec(VecType::U8, 16)
    }

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
            Type::Vec(ty, size) => ty.size() * *size,
            Type::Void => 0,
        }
    }

    pub fn uscalar_from_size(size: usize) -> Type {
        match size {
            1 => Type::U8,
            2 => Type::U16,
            4 => Type::U32,
            8 => Type::U64,
            _ => unreachable!("Invalid size: {}", size),
        }
    }

    pub fn iscalar_from_size(size: usize) -> Type {
        match size {
            1 => Type::I8,
            2 => Type::I16,
            4 => Type::I32,
            8 => Type::I64,
            _ => unreachable!("Invalid size: {}", size),
        }
    }

    pub fn is_unsigned(&self) -> bool {
        match self {
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => true,
            _ => false,
        }
    }

    pub fn gen_mask(&self) -> u64 {
        match self {
            Type::Bool => 0b1u64,
            Type::U8 | Type::I8 => u8::max_value() as u64,
            Type::U16 | Type::I16 => u16::max_value() as u64,
            Type::U32 | Type::I32 | Type::F32 => u32::max_value() as u64,
            Type::U64 | Type::I64 | Type::F64 => u64::max_value(),
            Type::Void => 0b0,
            _ => unimplemented!("gen_mask for {:?}", self),
        }
    }
}
