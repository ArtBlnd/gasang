use crate::ir::{Ir, Type};
use crate::register::RegId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operand {
    Ir(Box<Ir>),
    Register(RegId, Type),
    Immediate(u64, Type),
    Eip,
}

impl Operand {
    pub fn get_type(&self) -> Type {
        match self {
            Operand::Ir(ir) => ir.get_type(),
            Operand::Register(_, t) => *t,
            Operand::Immediate(_, t) => *t,
            Operand::Eip => Type::U64,
        }
    }

    pub fn validate(&self) -> bool {
        match self {
            Operand::Ir(ir) => ir.validate(),
            Operand::Register(_, _) => true,
            Operand::Immediate(_, _) => true,
            Operand::Eip => true,
        }
    }
}
