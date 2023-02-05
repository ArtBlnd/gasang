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

    Load(Type, Operand),

    And(Type, Operand, Operand),
    Or(Type, Operand, Operand),
    Xor(Type, Operand, Operand),

    LShl(Type, Operand, Operand),
    LShr(Type, Operand, Operand),
    AShr(Type, Operand, Operand),
    Rotr(Type, Operand, Operand),

    ZextCast(Type, Operand),
    SextCast(Type, Operand),
    BitCast(Type, Operand),

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

            Ir::Load(t, _) => *t,

            Ir::LShl(t, _, _) => *t,
            Ir::LShr(t, _, _) => *t,
            Ir::AShr(t, _, _) => *t,
            Ir::Rotr(t, _, _) => *t,

            Ir::ZextCast(t, _) => *t,
            Ir::SextCast(t, _) => *t,
            Ir::BitCast(t, _) => *t,

            Ir::And(t, _, _) => *t,
            Ir::Or(t, _, _) => *t,
            Ir::Xor(t, _, _) => *t,
            Ir::Value(op) => op.get_type(),
            Ir::Nop => Type::Void,
        }
    }

    pub fn validate(&self) -> bool {
        match self {
            Ir::Add(t, op1, op2)
            | Ir::Sub(t, op1, op2)
            | Ir::Mul(t, op1, op2)
            | Ir::Div(t, op1, op2)
            | Ir::And(t, op1, op2)
            | Ir::Or(t, op1, op2)
            | Ir::Xor(t, op1, op2) => {
                op1.validate()
                    && op2.validate()
                    && op1.get_type() == op2.get_type()
                    && op1.get_type() == *t
            }
            Ir::Load(t, op) => {
                op.validate()
                    && match op.get_type() {
                        Type::U8 | Type::U16 | Type::U32 | Type::U64 => true,
                        _ => false,
                    }
                    && *t == Type::U64
            }
            Ir::LShl(t, op1, op2)
            | Ir::LShr(t, op1, op2)
            | Ir::AShr(t, op1, op2)
            | Ir::Rotr(t, op1, op2) => {
                op1.validate()
                    && op2.validate()
                    && op1.get_type() == op2.get_type()
                    && op1.get_type() == *t
            }
            Ir::ZextCast(_, op) => op.validate(),
            Ir::SextCast(_, op) => op.validate(),
            Ir::BitCast(_, op) => op.validate(),
            Ir::Value(op) => op.validate(),
            Ir::Nop => true,
        }
    }
}
