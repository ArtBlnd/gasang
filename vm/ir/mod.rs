mod block;
pub use block::*;
mod operand;
pub use operand::*;
mod ty;
pub use ty::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ir {
    Add(Type, Operand, Operand),
    Sub(Type, Operand, Operand),
    Mul(Type, Operand, Operand),
    Div(Type, Operand, Operand),

    Addc(Type, Operand, Operand),
    Subc(Type, Operand, Operand),

    And(Type, Operand, Operand),
    Or(Type, Operand, Operand),
    Xor(Type, Operand, Operand),
    Not(Type, Operand),

    LShl(Type, Operand, Operand),
    LShr(Type, Operand, Operand),
    AShr(Type, Operand, Operand),
    Rotr(Type, Operand, Operand),

    Load(Type, Operand),

    ZextCast(Type, Operand),
    SextCast(Type, Operand),
    BitCast(Type, Operand),

    If(Type, Operand, Operand, Operand), // If(ret_type, condition, if_true, if_false)
    CmpEq(Operand, Operand),             // Equal
    CmpNe(Operand, Operand),             // Not equal
    CmpGt(Operand, Operand),             // Greater than
    CmpLt(Operand, Operand),             // Less than
    Value(Operand),
    Nop,
}

impl Ir {
    pub fn get_type(&self) -> Type {
        match self {
            Ir::Add(t, _, _) => *t,
            Ir::Sub(t, _, _) => *t,
            Ir::Mul(t, _, _) => *t,
            Ir::Div(t, _, _) => *t,

            Ir::Addc(t, _, _) => *t,
            Ir::Subc(t, _, _) => *t,

            Ir::And(t, _, _) => *t,
            Ir::Or(t, _, _) => *t,
            Ir::Xor(t, _, _) => *t,
            Ir::Not(t, _) => *t,

            Ir::LShl(t, _, _) => *t,
            Ir::LShr(t, _, _) => *t,
            Ir::AShr(t, _, _) => *t,
            Ir::Rotr(t, _, _) => *t,

            Ir::Load(t, _) => *t,

            Ir::ZextCast(t, _) => *t,
            Ir::SextCast(t, _) => *t,
            Ir::BitCast(t, _) => *t,

            Ir::Value(op) => op.get_type(),
            Ir::Nop => Type::Void,

            Ir::If(t, _, _, _) => *t,
            Ir::CmpEq(_, _) | Ir::CmpNe(_, _) | Ir::CmpGt(_, _) | Ir::CmpLt(_, _) => Type::Bool,
        }
    }
}
