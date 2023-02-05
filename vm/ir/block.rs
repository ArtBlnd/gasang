use crate::ir::{Ir, Type};

use crate::register::RegId;

#[derive(Clone, Debug)]
pub enum BlockDestination {
    Flags,
    Eip,
    GprRegister(RegId),
    FprRegister(RegId),
    Memory(u64),
    None,
}

#[derive(Clone, Debug)]
pub struct Block {
    ir_root: Ir,
    ir_dest: BlockDestination,

    original_size: usize,
}

impl Block {
    pub fn new(ir_root: Ir, ir_dest: BlockDestination, original_size: usize) -> Self {
        Self {
            ir_root,
            ir_dest,
            original_size,
        }
    }

    pub fn ir_root(&self) -> &Ir {
        &self.ir_root
    }

    pub fn ir_dest(&self) -> BlockDestination {
        self.ir_dest.clone()
    }

    pub fn original_size(&self) -> usize {
        self.original_size
    }

    pub fn validate(&self) -> bool {
        match self.ir_dest {
            BlockDestination::Flags => {
                self.ir_root().get_type() == Type::U64 && self.ir_root().validate()
            }
            BlockDestination::Eip => {
                self.ir_root().get_type() == Type::U64 && self.ir_root().validate()
            }
            BlockDestination::GprRegister(_) => match self.ir_root().get_type() {
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64 => true && self.ir_root().validate(),
                _ => false,
            },
            BlockDestination::FprRegister(_) => match self.ir_root().get_type() {
                Type::F32 | Type::F64 => true && self.ir_root().validate(),
                _ => false,
            },
            BlockDestination::Memory(_) => match self.ir_root().get_type() {
                Type::U8 | Type::U16 | Type::U32 | Type::U64 => true && self.ir_root().validate(),
                _ => false,
            },
            BlockDestination::None => true,
        }
    }
}
