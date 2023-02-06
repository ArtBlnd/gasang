use crate::codegen::flag_policy::FlagPolicy;
use crate::codegen::interpret::InterpretExeuctableBlock;
use crate::codegen::Codegen;
use crate::error::CodegenError;
use crate::ir::{Ir, IrBlock, Operand, Type};
use crate::VmState;

mod code;
pub use code::*;
mod function;
pub use function::*;
use smallvec::SmallVec;

use std::sync::Arc;

pub struct InterpretCodegen {
    flag_policy: Arc<dyn FlagPolicy>,
}

impl InterpretCodegen {
    pub fn new<F>(flag_policy: F) -> Self
    where
        F: FlagPolicy + 'static,
    {
        Self {
            flag_policy: Arc::new(flag_policy),
        }
    }
}

impl Codegen for InterpretCodegen {
    type Executable = CodeBlock;

    fn compile(&self, blocks: Vec<IrBlock>) -> Result<Self::Executable, CodegenError> {
        let mut codes = Vec::new();
        let mut sizes = Vec::new();

        for block in blocks {
            let size = block.original_size();
            let mut exec = SmallVec::new();

            for block_item in block.items() {
                exec.push(InterpretExeuctableBlock {
                    code: unsafe { compile_ir(block_item.root(), self.flag_policy.clone())? },
                    code_type: block_item.root().get_type(),
                    code_dest: block_item.dest().clone(),

                    restore_flag: block.restore_flag(),
                });
            }

            codes.push(exec);
            sizes.push(size);
        }

        Ok(CodeBlock { codes, sizes })
    }
}

unsafe fn compile_ir(
    ir: &Ir,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    // Optimiations
    match ir {
        Ir::Add(Type::U64, Operand::Ip, Operand::Immediate(imm, Type::I64)) => {
            let imm = *imm;
            return Ok(Box::new(move |ctx| (ctx.ip() as i64 + imm as i64) as u64));
        }
        Ir::Add(Type::U64, Operand::Ip, Operand::Immediate(imm, Type::U64)) => {
            let imm = *imm;
            return Ok(Box::new(move |ctx| ctx.ip() + imm));
        }
        Ir::Value(Operand::Ir(ir)) => return compile_ir(ir, flag_policy.clone()),
        Ir::Value(Operand::Immediate(imm, t)) => {
            let imm = *imm;
            let t = *t;

            return Ok(Box::new(move |_ctx| imm & type_mask(t)));
        }
        Ir::Value(Operand::Register(reg_id, t)) => {
            let reg = *reg_id;
            let t = *t;

            return Ok(Box::new(move |ctx| ctx.gpr(reg).get() & type_mask(t)));
        }
        Ir::LShr(t, op1, Operand::Immediate(value, t1)) => {
            assert!(t == t1, "Type mismatch");
            let op1 = compile_op(op1, flag_policy.clone())?;
            let value = *value;
            let t = *t;

            return Ok(Box::new(move |ctx| (op1(ctx) >> value) & type_mask(t)));
        }
        Ir::And(Type::U64, Operand::Flag, Operand::Immediate(imm, Type::U64)) => {
            let imm = *imm;
            return Ok(Box::new(move |ctx| ctx.flag() & imm));
        }
        Ir::And(t, Operand::Immediate(imm1, t1), Operand::Immediate(imm2, t2)) => {
            assert!(t1 == t2 && t1 == t, "Type mismatch");
            let imm1 = *imm1;
            let imm2 = *imm2;
            let t = *t;

            return Ok(Box::new(move |_ctx| (imm1 & imm2) & type_mask(t)));
        }
        Ir::If(t, Operand::Immediate(imm, t1), op1, op2) => {
            assert!(*t1 == Type::Bool, "Type mismatch");
            assert!(op1.get_type() == op2.get_type(), "Type mismatch");
            let imm = *imm;
            let t = *t;
            let op1 = compile_op(op1, flag_policy.clone())?;
            let op2 = compile_op(op2, flag_policy.clone())?;

            return Ok(Box::new(move |ctx| {
                type_mask(t) & if imm != 0 { op1(ctx) } else { op2(ctx) }
            }));
        }

        _ => {}
    };

    match ir {
        Ir::Add(t, op1, op2) => gen_add(t, op1, op2, flag_policy),
        Ir::Sub(t, op1, op2) => gen_sub(t, op1, op2, flag_policy),
        Ir::Mul(t, op1, op2) => gen_mul(t, op1, op2, flag_policy),
        Ir::Div(t, op1, op2) => gen_div(t, op1, op2, flag_policy),
        Ir::Addc(t, op1, op2) => gen_addc(t, op1, op2, flag_policy),
        Ir::Subc(t, op1, op2) => gen_subc(t, op1, op2, flag_policy),

        Ir::And(t, op1, op2) => gen_and(t, op1, op2, flag_policy),
        Ir::Or(t, op1, op2) => gen_or(t, op1, op2, flag_policy),
        Ir::Xor(t, op1, op2) => gen_xor(t, op1, op2, flag_policy),
        Ir::Not(t, op) => gen_not(t, op, flag_policy),

        Ir::LShl(t, op1, op2) => gen_lshl(t, op1, op2, flag_policy),
        Ir::LShr(t, op1, op2) => gen_lshr(t, op1, op2, flag_policy),
        Ir::AShr(t, op1, op2) => gen_ashr(t, op1, op2, flag_policy),
        Ir::Rotr(t, op1, op2) => gen_rotr(t, op1, op2, flag_policy),

        Ir::Load(t, op) => gen_load(t, op, flag_policy),

        Ir::ZextCast(t, op) => gen_zext_cast(t, op, flag_policy),
        Ir::SextCast(t, op) => gen_sext_cast(t, op, flag_policy),
        Ir::BitCast(t, op) => gen_bit_cast(t, op, flag_policy),

        Ir::Value(op) => compile_op(op, flag_policy.clone()),
        Ir::Nop => Ok(Box::new(|_| 0)),

        Ir::If(t, cond, if_true, if_false) => gen_if(t, cond, if_true, if_false, flag_policy),
        Ir::CmpEq(op1, op2) => gen_cmp_eq(op1, op2, flag_policy),
        Ir::CmpNe(op1, op2) => gen_cmp_ne(op1, op2, flag_policy),
        Ir::CmpGt(op1, op2) => gen_cmp_gt(op1, op2, flag_policy),
        Ir::CmpLt(op1, op2) => gen_cmp_lt(op1, op2, flag_policy),
    }
}

unsafe fn compile_op(
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    Ok(match op {
        Operand::Ir(ir) => compile_ir(ir, flag_policy.clone())?,
        Operand::Register(id, t) => {
            let id = *id;
            let t = *t;
            Box::new(move |vm: &mut VmState| vm.gpr(id).get() & type_mask(t))
        }
        Operand::Immediate(imm, t) => {
            let imm = *imm;
            let t = *t;
            Box::new(move |_| imm & type_mask(t))
        }
        Operand::VoidIr(ir) => compile_ir(ir, flag_policy.clone())?,
        Operand::Ip => Box::new(move |ctx| ctx.ip()),
        Operand::Flag => Box::new(move |ctx| ctx.flag()),
    })
}

const fn type_mask(t: Type) -> u64 {
    match t {
        Type::Bool => 0b1u64,
        Type::U8 | Type::I8 => u8::max_value() as u64,
        Type::U16 | Type::I16 => u16::max_value() as u64,
        Type::U32 | Type::I32 | Type::F32 => u32::max_value() as u64,
        Type::U64 | Type::I64 | Type::F64 => u64::max_value(),
        Type::Void => 0b0,
    }
}

unsafe fn gen_add(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 => Box::new(move |ctx| (lhs.execute(ctx) as u8 + rhs.execute(ctx) as u8) as u64),
        Type::U16 => {
            Box::new(move |ctx| (lhs.execute(ctx) as u16 + rhs.execute(ctx) as u16) as u64)
        }
        Type::U32 => {
            Box::new(move |ctx| (lhs.execute(ctx) as u32 + rhs.execute(ctx) as u32) as u64)
        }
        Type::U64 => Box::new(move |ctx| lhs.execute(ctx) + rhs.execute(ctx)),
        Type::I8 => Box::new(move |ctx| (lhs.execute(ctx) as i8 + rhs.execute(ctx) as i8) as u64),
        Type::I16 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i16 + rhs.execute(ctx) as i16) as u64)
        }
        Type::I32 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i32 + rhs.execute(ctx) as i32) as u64)
        }
        Type::I64 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i64 + rhs.execute(ctx) as i64) as u64)
        }
        Type::F32 => Box::new(move |ctx| {
            (f32::from_bits(lhs.execute(ctx) as u32) + f32::from_bits(rhs.execute(ctx) as u32))
                .to_bits() as u64
        }),
        Type::F64 => Box::new(move |ctx| {
            (f64::from_bits(lhs.execute(ctx)) + f64::from_bits(rhs.execute(ctx))).to_bits()
        }),
        Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_sub(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 => Box::new(move |ctx| (lhs.execute(ctx) as u8 - rhs.execute(ctx) as u8) as u64),
        Type::U16 => {
            Box::new(move |ctx| (lhs.execute(ctx) as u16 - rhs.execute(ctx) as u16) as u64)
        }
        Type::U32 => {
            Box::new(move |ctx| (lhs.execute(ctx) as u32 - rhs.execute(ctx) as u32) as u64)
        }
        Type::U64 => Box::new(move |ctx| lhs.execute(ctx) - rhs.execute(ctx)),
        Type::I8 => Box::new(move |ctx| (lhs.execute(ctx) as i8 - rhs.execute(ctx) as i8) as u64),
        Type::I16 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i16 - rhs.execute(ctx) as i16) as u64)
        }
        Type::I32 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i32 - rhs.execute(ctx) as i32) as u64)
        }
        Type::I64 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i64 - rhs.execute(ctx) as i64) as u64)
        }
        Type::F32 => Box::new(move |ctx| {
            (f32::from_bits(lhs.execute(ctx) as u32) - f32::from_bits(rhs.execute(ctx) as u32))
                .to_bits() as u64
        }),
        Type::F64 => Box::new(move |ctx| {
            (f64::from_bits(lhs.execute(ctx)) - f64::from_bits(rhs.execute(ctx))).to_bits()
        }),
        Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_mul(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 => Box::new(move |ctx| (lhs.execute(ctx) as u8 * rhs.execute(ctx) as u8) as u64),
        Type::U16 => {
            Box::new(move |ctx| (lhs.execute(ctx) as u16 * rhs.execute(ctx) as u16) as u64)
        }
        Type::U32 => {
            Box::new(move |ctx| (lhs.execute(ctx) as u32 * rhs.execute(ctx) as u32) as u64)
        }
        Type::U64 => Box::new(move |ctx| lhs.execute(ctx) * rhs.execute(ctx)),
        Type::I8 => Box::new(move |ctx| (lhs.execute(ctx) as i8 * rhs.execute(ctx) as i8) as u64),
        Type::I16 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i16 * rhs.execute(ctx) as i16) as u64)
        }
        Type::I32 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i32 * rhs.execute(ctx) as i32) as u64)
        }
        Type::I64 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i64 * rhs.execute(ctx) as i64) as u64)
        }
        Type::F32 => Box::new(move |ctx| {
            (f32::from_bits(lhs.execute(ctx) as u32) * f32::from_bits(rhs.execute(ctx) as u32))
                .to_bits() as u64
        }),
        Type::F64 => Box::new(move |ctx| {
            (f64::from_bits(lhs.execute(ctx)) * f64::from_bits(rhs.execute(ctx))).to_bits()
        }),

        Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_div(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 => Box::new(move |ctx| (lhs.execute(ctx) as u8 / rhs.execute(ctx) as u8) as u64),
        Type::U16 => {
            Box::new(move |ctx| (lhs.execute(ctx) as u16 / rhs.execute(ctx) as u16) as u64)
        }
        Type::U32 => {
            Box::new(move |ctx| (lhs.execute(ctx) as u32 / rhs.execute(ctx) as u32) as u64)
        }
        Type::U64 => Box::new(move |ctx| lhs.execute(ctx) / rhs.execute(ctx)),
        Type::I8 => Box::new(move |ctx| (lhs.execute(ctx) as i8 / rhs.execute(ctx) as i8) as u64),
        Type::I16 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i16 / rhs.execute(ctx) as i16) as u64)
        }
        Type::I32 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i32 / rhs.execute(ctx) as i32) as u64)
        }
        Type::I64 => {
            Box::new(move |ctx| (lhs.execute(ctx) as i64 / rhs.execute(ctx) as i64) as u64)
        }
        Type::F32 => Box::new(move |ctx| {
            (f32::from_bits(lhs.execute(ctx) as u32) / f32::from_bits(rhs.execute(ctx) as u32))
                .to_bits() as u64
        }),
        Type::F64 => Box::new(move |ctx| {
            (f64::from_bits(lhs.execute(ctx)) / f64::from_bits(rhs.execute(ctx))).to_bits()
        }),

        Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_addc(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = t.clone();

    Ok(match t {
        Type::U8 => Box::new(move |ctx| {
            let carry_in: u8 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.add_carry(t, lhs, rhs, ctx);

            (lhs as u8 + rhs as u8 + carry_in) as u64
        }),
        Type::U16 => Box::new(move |ctx| {
            let carry_in: u16 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.add_carry(t, lhs, rhs, ctx);

            (lhs as u16 + rhs as u16 + carry_in) as u64
        }),
        Type::U32 => Box::new(move |ctx| {
            let carry_in: u32 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.add_carry(t, lhs, rhs, ctx);

            (lhs as u32 + rhs as u32 + carry_in) as u64
        }),
        Type::U64 => Box::new(move |ctx| {
            let carry_in: u64 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.add_carry(t, lhs, rhs, ctx);

            lhs + rhs + carry_in
        }),
        Type::I8 => Box::new(move |ctx| {
            let carry_in: i8 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.add_carry(t, lhs, rhs, ctx);

            (lhs as i8 + rhs as i8 + carry_in) as u64
        }),
        Type::I16 => Box::new(move |ctx| {
            let carry_in: i16 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.add_carry(t, lhs, rhs, ctx);

            (lhs as i16 + rhs as i16 + carry_in) as u64
        }),
        Type::I32 => Box::new(move |ctx| {
            let carry_in: i32 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.add_carry(t, lhs, rhs, ctx);

            (lhs as i32 + rhs as i32 + carry_in) as u64
        }),
        Type::I64 => Box::new(move |ctx| {
            let carry_in: i64 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.add_carry(t, lhs, rhs, ctx);

            (lhs as i64 + rhs as i64 + carry_in) as u64
        }),
        Type::F32 | Type::F64 => return Err(CodegenError::InvalidType),
        Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_subc(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = t.clone();
    Ok(match t {
        Type::U8 => Box::new(move |ctx| {
            let carry_in: u8 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.sub_carry(t, lhs, rhs, ctx);

            (lhs as u8 - rhs as u8 + carry_in) as u64
        }),
        Type::U16 => Box::new(move |ctx| {
            let carry_in: u16 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.sub_carry(t, lhs, rhs, ctx);

            (lhs as u16 - rhs as u16 + carry_in) as u64
        }),
        Type::U32 => Box::new(move |ctx| {
            let carry_in: u32 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.sub_carry(t, lhs, rhs, ctx);

            (lhs as u32 - rhs as u32 + carry_in) as u64
        }),
        Type::U64 => Box::new(move |ctx| {
            let carry_in: u64 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.sub_carry(t, lhs, rhs, ctx);

            lhs - rhs + carry_in
        }),
        Type::I8 => Box::new(move |ctx| {
            let carry_in: i8 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.sub_carry(t, lhs, rhs, ctx);

            (lhs as i8 - rhs as i8 + carry_in) as u64
        }),
        Type::I16 => Box::new(move |ctx| {
            let carry_in: i16 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.sub_carry(t, lhs, rhs, ctx);

            (lhs as i16 - rhs as i16 + carry_in) as u64
        }),
        Type::I32 => Box::new(move |ctx| {
            let carry_in: i32 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.sub_carry(t, lhs, rhs, ctx);

            (lhs as i32 - rhs as i32 + carry_in) as u64
        }),
        Type::I64 => Box::new(move |ctx| {
            let carry_in: i64 = flag_policy.carry(ctx).into();
            let lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            flag_policy.sub_carry(t, lhs, rhs, ctx);

            (lhs as i64 - rhs as i64 + carry_in) as u64
        }),
        Type::F32 | Type::F64 => return Err(CodegenError::InvalidType),
        Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_lshl(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;
    Ok(match t {
        Type::U8 => Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as u8 as u64),
        Type::U16 => Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as u16 as u64),
        Type::U32 => Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as u32 as u64),
        Type::U64 => Box::new(move |ctx| lhs.execute(ctx) << rhs.execute(ctx)),
        Type::I8 => Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as i8 as u64),
        Type::I16 => Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as i16 as u64),
        Type::I32 => Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as i32 as u64),
        Type::I64 => Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) as i64 as u64),
        Type::F32 | Type::F64 | Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_lshr(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u8 as u64),
        Type::U16 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u16 as u64),
        Type::U32 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u32 as u64),
        Type::U64 => Box::new(move |ctx| lhs.execute(ctx) >> rhs.execute(ctx)),
        Type::I8 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as i8 as u64),
        Type::I16 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as i16 as u64),
        Type::I32 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as i32 as u64),
        Type::I64 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as i64 as u64),
        Type::F32 | Type::F64 | Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_ashr(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u8 as u64),
        Type::U16 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u16 as u64),
        Type::U32 => Box::new(move |ctx| (lhs.execute(ctx) >> rhs.execute(ctx)) as u32 as u64),
        Type::U64 => Box::new(move |ctx| lhs.execute(ctx) >> rhs.execute(ctx)),
        Type::I8 => Box::new(move |ctx| (lhs.execute(ctx) as i8 >> rhs.execute(ctx)) as u64),
        Type::I16 => Box::new(move |ctx| (lhs.execute(ctx) as i16 >> rhs.execute(ctx)) as u64),
        Type::I32 => Box::new(move |ctx| (lhs.execute(ctx) as i32 >> rhs.execute(ctx)) as u64),
        Type::I64 => Box::new(move |ctx| (lhs.execute(ctx) as i64 >> rhs.execute(ctx)) as u64),
        Type::F32 | Type::F64 | Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_rotr(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 => Box::new(move |ctx| {
            ((lhs.execute(ctx) as u8).rotate_right(rhs.execute(ctx) as u32)) as u64
        }),
        Type::U16 => Box::new(move |ctx| {
            ((lhs.execute(ctx) as u16).rotate_right(rhs.execute(ctx) as u32)) as u64
        }),
        Type::U32 => Box::new(move |ctx| {
            ((lhs.execute(ctx) as u32).rotate_right(rhs.execute(ctx) as u32)) as u64
        }),
        Type::U64 => Box::new(move |ctx| lhs.execute(ctx).rotate_right(rhs.execute(ctx) as u32)),
        Type::I8 => Box::new(move |ctx| {
            ((lhs.execute(ctx) as i8).rotate_right(rhs.execute(ctx) as u32)) as u64
        }),
        Type::I16 => Box::new(move |ctx| {
            ((lhs.execute(ctx) as i16).rotate_right(rhs.execute(ctx) as u32)) as u64
        }),
        Type::I32 => Box::new(move |ctx| {
            ((lhs.execute(ctx) as i32).rotate_right(rhs.execute(ctx) as u32)) as u64
        }),
        Type::I64 => Box::new(move |ctx| {
            ((lhs.execute(ctx) as i64).rotate_right(rhs.execute(ctx) as u32)) as u64
        }),
        Type::F32 | Type::F64 | Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_load(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let op = compile_op(op, flag_policy.clone())?;
    Ok(match t {
        Type::Bool => Box::new(move |ctx| {
            let var = op.execute(ctx);
            (ctx.mem(var).read_u8().unwrap() & 0b1) as u64
        }),
        Type::U8 | Type::I8 => Box::new(move |ctx| {
            let var = op.execute(ctx);
            ctx.mem(var).read_u8().unwrap() as u64
        }),
        Type::U16 | Type::I16 => Box::new(move |ctx| {
            let var = op.execute(ctx);
            ctx.mem(var).read_u16().unwrap() as u64
        }),
        Type::U32 | Type::I32 | Type::F32 => Box::new(move |ctx| {
            let var = op.execute(ctx);
            ctx.mem(var).read_u32().unwrap() as u64
        }),
        Type::U64 | Type::I64 | Type::F64 => Box::new(move |ctx| {
            let var = op.execute(ctx);
            ctx.mem(var).read_u64().unwrap()
        }),
        Type::Void => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_zext_cast(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let op = compile_op(op, flag_policy.clone())?;
    Ok(match t {
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => Box::new(move |ctx| op.execute(ctx)),
        Type::F32 | Type::F64 | Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_sext_cast(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let from = op.get_type();
    let to = type_mask(*t);
    let op = compile_op(op, flag_policy.clone())?;

    Ok(match from {
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => op,
        Type::I8 => Box::new(move |ctx| {
            let v: i64 = (op.execute(ctx) as i8).into();
            v as u64 & to
        }),
        Type::I16 => Box::new(move |ctx| {
            let v: i64 = (op.execute(ctx) as i16).into();
            v as u64 & to
        }),
        Type::I32 => Box::new(move |ctx| {
            let v: i64 = (op.execute(ctx) as i32).into();
            v as u64 & to
        }),
        Type::I64 => Box::new(move |ctx| {
            let v: i64 = op.execute(ctx) as i64;
            v as u64 & to
        }),
        Type::F32 | Type::F64 | Type::Void | Type::Bool => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_bit_cast(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let op = compile_op(op, flag_policy.clone())?;
    let to = type_mask(*t);

    Ok(Box::new(move |ctx| unsafe { op.execute(ctx) & to }))
}

unsafe fn gen_and(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = *t;
    Ok(match t {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => Box::new(move |ctx| (lhs.execute(ctx) & rhs.execute(ctx)) & type_mask(t)),
        Type::F32 | Type::F64 | Type::Void => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_or(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;
    let t = *t;
    Ok(match t {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => Box::new(move |ctx| (lhs.execute(ctx) | rhs.execute(ctx)) & type_mask(t)),
        Type::F32 | Type::F64 | Type::Void => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_xor(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = *t;

    Ok(match t {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => Box::new(move |ctx| (lhs.execute(ctx) ^ rhs.execute(ctx)) & type_mask(t)),
        Type::F32 | Type::F64 | Type::Void => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_not(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let op = compile_op(op, flag_policy.clone())?;

    let t = *t;

    Ok(match t {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => Box::new(move |ctx| (!op.execute(ctx) & type_mask(t)) as u64),
        Type::F32 | Type::F64 | Type::Void => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_if(
    t: &Type,
    cond: &Operand,
    if_true: &Operand,
    if_false: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    assert_eq!(cond.get_type(), Type::Bool);

    let cond = compile_op(cond, flag_policy.clone())?;
    let if_true = compile_op(if_true, flag_policy.clone())?;
    let if_false = compile_op(if_false, flag_policy.clone())?;

    let t = *t;
    Ok(match t {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => Box::new(move |ctx| {
            type_mask(t)
                & if cond.execute(ctx) != 0 {
                    if_true.execute(ctx)
                } else {
                    if_false.execute(ctx)
                }
        }),
        Type::F32 | Type::F64 | Type::Void => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_cmp_eq(
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = op1.get_type();
    assert_eq!(op1.get_type(), op2.get_type());

    Ok(match t {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::F32
        | Type::F64 => Box::new(move |ctx| {
            (lhs.execute(ctx) & type_mask(t) == rhs.execute(ctx) & type_mask(t)) as u64
        }),
        Type::Void => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_cmp_ne(
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = op1.get_type();
    assert_eq!(op1.get_type(), op2.get_type());

    Ok(match t {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::F32
        | Type::F64 => Box::new(move |ctx| {
            (lhs.execute(ctx) & type_mask(t) != rhs.execute(ctx) & type_mask(t)) as u64
        }),
        Type::Void => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_cmp_gt(
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = op1.get_type();
    assert_eq!(op1.get_type(), op2.get_type());

    Ok(match t {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::F32
        | Type::F64 => Box::new(move |ctx| {
            (lhs.execute(ctx) & type_mask(t) > rhs.execute(ctx) & type_mask(t)) as u64
        }),
        Type::Void => return Err(CodegenError::InvalidType),
    })
}

unsafe fn gen_cmp_lt(
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn InterpretFunc>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = op1.get_type();
    assert_eq!(op1.get_type(), op2.get_type());

    Ok(match t {
        Type::Bool
        | Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64
        | Type::F32
        | Type::F64 => Box::new(move |ctx| {
            (lhs.execute(ctx) & type_mask(t) < rhs.execute(ctx) & type_mask(t)) as u64
        }),
        Type::Void => return Err(CodegenError::InvalidType),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::codegen::flag_policy::DummyFlagPolicy;
    use crate::image::Image;
    use crate::interrupt::NoModel;
    use crate::vm_builder::VmBuilder;

    #[test]
    fn test_compile_simple_imm_ir() {
        let image = Image::from_image(vec![]);
        let mut vm = VmBuilder::new(&image).build(0, NoModel);

        let flag_policy = Arc::new(DummyFlagPolicy) as Arc<dyn FlagPolicy>;

        //Test UADD
        let ir = Ir::Add(
            Type::U64,
            Operand::Immediate(30, Type::U64),
            Operand::Immediate(50, Type::U64),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result, 80);

        //Test IADD
        let ir = Ir::Add(
            Type::I64,
            Operand::Immediate((-10i64) as u64, Type::I64),
            Operand::Immediate(10, Type::I64),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result, 0);

        //Test USUB
        let ir = Ir::Sub(
            Type::U8,
            Operand::Immediate(10, Type::U8),
            Operand::Immediate(9, Type::U8),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result, 1_u64);

        //Test subtract minus value ISUB
        let ir = Ir::Sub(
            Type::I16,
            Operand::Immediate(10, Type::I16),
            Operand::Immediate((-10i16) as u64, Type::I16),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result, 20);

        //Test IMUL
        let ir = Ir::Mul(
            Type::I32,
            Operand::Immediate(10, Type::I32),
            Operand::Immediate((-10i64) as u64, Type::I32),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result as i64, -100);

        //Test UMUL-F64
        let ir = Ir::Mul(
            Type::F64,
            Operand::Immediate((10f64).to_bits(), Type::F64),
            Operand::Immediate((4.5f64).to_bits(), Type::F64),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert!(f64::from_bits(result) > 44.9 && f64::from_bits(result) < 45.1);

        //Test IDIV
        let ir = Ir::Div(
            Type::I64,
            Operand::Immediate(10_u64, Type::I64),
            Operand::Immediate((-5i64) as u64, Type::I64),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result as i64, -2);

        //Test UDIV-F64
        let ir = Ir::Div(
            Type::F64,
            Operand::Immediate((10f64).to_bits(), Type::F64),
            Operand::Immediate((3.3f64).to_bits(), Type::F64),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert!(f64::from_bits(result) > 3.0 && f64::from_bits(result) < 3.1);

        //Test LSHL
        let ir = Ir::LShl(
            Type::U8,
            Operand::Immediate(0b1111_1111, Type::U8),
            Operand::Immediate(4, Type::U8),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result, 0b1111_0000);

        //Test LSHR
        let ir = Ir::LShr(
            Type::U8,
            Operand::Immediate(0b0000_1111, Type::U8),
            Operand::Immediate(4, Type::U8),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result, 0b0000_0000);

        //Test Rotr
        let ir = Ir::Rotr(
            Type::U8,
            Operand::Immediate(0b1010_1111, Type::U8),
            Operand::Immediate(2, Type::U8),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result, 0b1110_1011);

        //Test Zext
        let ir = Ir::ZextCast(Type::U16, Operand::Immediate((-10i8) as u64, Type::U16));
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result as u16, 65526);

        //Test Sext
        let ir = Ir::SextCast(Type::I64, Operand::Immediate(0xFF, Type::I8));
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result as i64, -1);

        //Test Sext-32
        let ir = Ir::SextCast(Type::I32, Operand::Immediate(0xFF, Type::I8));
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result as i64, u32::max_value() as i64);
    }
}
