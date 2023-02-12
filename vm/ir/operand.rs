use crate::ir::{Ir, Type};
use crate::register::RegId;
use crate::vm_state::VmInfo;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operand {
    Ir(Box<Ir>),
    VoidIr(Box<Ir>),
    Gpr(Type, RegId),
    Fpr(Type, RegId),
    Immediate(Type, u64),
    Ip,
    Flag,
    Dbg(String, Box<Operand>),
    VmInfo(VmInfo),
}

impl Operand {
    pub fn get_type(&self) -> Type {
        match self {
            Operand::Ir(ir) => ir.get_type(),
            Operand::VoidIr(_ir) => Type::Void,
            Operand::Gpr(t, _) => *t,
            Operand::Fpr(t, _) => *t,
            Operand::Immediate(t, _) => *t,
            Operand::Ip => Type::U64,
            Operand::Flag => Type::U64,
            Operand::Dbg(_, operand) => operand.get_type(),
            Operand::VmInfo(_) => Type::U64,
        }
    }

    pub fn ir(ir: Ir) -> Self {
        Operand::Ir(Box::new(ir))
    }

    pub fn void_ir(ir: Ir) -> Self {
        Operand::VoidIr(Box::new(ir))
    }

    pub const fn gpr(t: Type, reg: RegId) -> Self {
        Operand::Gpr(t, reg)
    }

    pub const fn imm(t: Type, imm: u64) -> Self {
        Operand::Immediate(t, imm)
    }

    pub fn dbg(msg: impl AsRef<str>, operand: Operand) -> Self {
        Operand::Dbg(msg.as_ref().to_string(), Box::new(operand))
    }
}
