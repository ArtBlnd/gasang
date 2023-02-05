use crate::ir::{Ir, Type};
use crate::register::RegId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operand {
    Ir(Box<Ir>),
    Register(RegId),
    Immediate(u64),
}

impl Operand {
    pub fn get_type(&self) -> Type {
        match self {
            Operand::Ir(ir) => ir.get_type(),
            Operand::Register(_) => Type::U64,
            Operand::Immediate(_) => Type::U64,
        }
    }

    pub fn validate(&self) -> bool {
        match self {
            Operand::Ir(ir) => ir.validate(),
            Operand::Register(_) => true,
            Operand::Immediate(_) => true,
        }
    }
}
