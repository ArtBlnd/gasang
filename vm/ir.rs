use slab::Slab;

pub type RegId = usize;
pub type FlagId = usize;

pub enum VMIR {
    Add,
    Sub,
    Mul,
    Div,

    Or,
    Xor,
    And,
    Not,

    Rhs,
    Lhs,
    Store,
    Load,
    Mov,
    MovIf,

    Push,
    Pop,

    Jump,
    JumpIf,

    Compare,
    Call,
    Ret,
    Nop,
    Halt,
}

pub enum Op {
    Register(RegId),
    Immediate(usize),
    Constant(usize),
    Flag(FlagId)
}