use crate::codegen::flag_policy::FlagPolicy;
use crate::codegen::*;
use crate::error::CodegenError;
use crate::ir::{Ir, IrBlock, Operand, Type};
use crate::VmState;

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
    type Code = Box<dyn CompiledCode>;

    fn compile(&self, ir: Ir) -> Self::Code {
        unsafe { 
            Box::new(compile_ir(&ir, self.flag_policy.clone()).unwrap())
        }
    }
}

unsafe fn compile_ir(
    ir: &Ir,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    // Optimiations
    match ir {
        Ir::Add(Type::U64, Operand::Ip, Operand::Immediate(Type::I64, imm)) => {
            let imm = *imm;
            return Ok(Box::new(move |ctx| (ctx.ip() as i64 + imm as i64) as u64));
        }
        Ir::Add(Type::U64, Operand::Ip, Operand::Immediate(Type::U64, imm)) => {
            let imm = *imm;
            return Ok(Box::new(move |ctx| ctx.ip() + imm));
        }
        Ir::Value(Operand::Ir(ir)) => return compile_ir(ir, flag_policy.clone()),
        Ir::Value(Operand::Immediate(t, imm)) => {
            let imm = *imm;
            let t = *t;

            return Ok(Box::new(move |_ctx| imm & t.gen_mask()));
        }
        Ir::Value(Operand::Register(t, reg)) => {
            let reg = *reg;
            let t = *t;

            return Ok(Box::new(move |ctx| ctx.gpr(reg).get() & t.gen_mask()));
        }
        Ir::LShr(t, op1, Operand::Immediate(t1, val)) => {
            assert!(t == t1, "Type mismatch");
            let op1 = compile_op(op1, flag_policy.clone())?;
            let val = *val;
            let t = *t;

            return Ok(Box::new(move |ctx| (op1(ctx) >> val) & t.gen_mask()));
        }
        Ir::And(Type::U64, Operand::Flag, Operand::Immediate(Type::U64, imm)) => {
            let imm = *imm;
            return Ok(Box::new(move |ctx| ctx.flag() & imm));
        }
        Ir::And(t, Operand::Immediate(t1, imm1), Operand::Immediate(t2, imm2)) => {
            assert!(t1 == t2 && t1 == t, "Type mismatch");
            let imm1 = *imm1;
            let imm2 = *imm2;
            let t = *t;

            return Ok(Box::new(move |_ctx| (imm1 & imm2) & t.gen_mask()));
        }
        Ir::If(t, Operand::Immediate(t1, imm), op1, op2) => {
            assert!(*t1 == Type::Bool, "Type mismatch");
            assert!(op1.get_type() == op2.get_type(), "Type mismatch");
            let imm = *imm;
            let t = *t;
            let op1 = compile_op(op1, flag_policy.clone())?;
            let op2 = compile_op(op2, flag_policy.clone())?;

            return Ok(Box::new(move |ctx| {
                t.gen_mask() & if imm != 0 { op1(ctx) } else { op2(ctx) }
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
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    Ok(match op {
        Operand::Ir(ir) => compile_ir(ir, flag_policy.clone())?,
        Operand::Register(t, reg) => {
            let reg = *reg;
            let t = *t;
            Box::new(move |vm| vm.gpr(reg).get() & t.gen_mask())
        }
        Operand::Immediate(t, imm) => {
            let imm = *imm;
            let t = *t;
            Box::new(move |_| imm & t.gen_mask())
        }
        Operand::VoidIr(ir) => compile_ir(ir, flag_policy.clone())?,
        Operand::Ip => Box::new(move |ctx| ctx.ip()),
        Operand::Flag => Box::new(move |ctx| ctx.flag()),
        Operand::Dbg(s, op) => {
            let op = compile_op(op, flag_policy.clone())?;
            let s = s.to_string();
            Box::new(move |ctx| {
                let val = op(ctx);
                println!("{s} {val}");
                val
            })
        }
        Operand::VmInfo(info_ty) => {
            todo!()
        }
    })
}

unsafe fn gen_add(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    let t1 = op1.get_type();
    let t2 = op2.get_type();

    if t1.is_float() {
        assert!(t2.is_float() && t1.size() == t2.size())
    } else {
        assert!(t1.is_scalar() && t2.is_scalar() && t1.size() == t2.size())
    }

    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = *t;
    Ok(match t {
        Type::U8
        | Type::I8
        | Type::U16
        | Type::I16
        | Type::U32
        | Type::I32
        | Type::U64
        | Type::I64 => Box::new(move |ctx| {
            (lhs.execute(ctx) as i64)
                .overflowing_add(rhs.execute(ctx) as i64)
                .0 as u64
                & t.gen_mask()
        }),
        Type::F32 => Box::new(move |ctx| {
            (f32::from_bits(lhs.execute(ctx) as u32) + f32::from_bits(rhs.execute(ctx) as u32))
                .to_bits() as u64
        }),
        Type::F64 => Box::new(move |ctx| {
            (f64::from_bits(lhs.execute(ctx)) + f64::from_bits(rhs.execute(ctx))).to_bits()
        }),
        Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_sub(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    let t1 = op1.get_type();
    let t2 = op2.get_type();

    if t1.is_float() {
        assert!(t2.is_float() && t1.size() == t2.size())
    } else {
        assert!(t1.is_scalar() && t2.is_scalar() && t1.size() == t2.size())
    }

    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = *t;
    Ok(match t {
        Type::U8
        | Type::I8
        | Type::U16
        | Type::I16
        | Type::U32
        | Type::I32
        | Type::U64
        | Type::I64 => Box::new(move |ctx| {
            (lhs.execute(ctx) as i64)
                .overflowing_sub(rhs.execute(ctx) as i64)
                .0 as u64
                & t.gen_mask() as u64
        }),

        Type::F32 => Box::new(move |ctx| {
            (f32::from_bits(lhs.execute(ctx) as u32) - f32::from_bits(rhs.execute(ctx) as u32))
                .to_bits() as u64
        }),
        Type::F64 => Box::new(move |ctx| {
            (f64::from_bits(lhs.execute(ctx)) - f64::from_bits(rhs.execute(ctx))).to_bits()
        }),
        Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_mul(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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

        Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_div(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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

        Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_addc(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    let t1 = op1.get_type();
    let t2 = op2.get_type();

    if t1.is_float() {
        assert!(t2.is_float() && t1.size() == t2.size())
    } else {
        assert!(t1.is_scalar() && t2.is_scalar() && t1.size() == t2.size())
    }

    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = *t;
    Ok(match t {
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => Box::new(move |ctx| {
            let lhs = lhs.execute(ctx) as i64;
            let rhs = rhs.execute(ctx) as i64;

            flag_policy.add_carry(t, lhs as u64, rhs as u64, ctx);

            lhs.overflowing_add(rhs).0 as u64 & t.gen_mask()
        }),
        Type::F32 | Type::F64 => unreachable!("invalid type: {:?}", t),
        Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_subc(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    let t1 = op1.get_type();
    let t2 = op2.get_type();

    if t1.is_float() {
        assert!(t2.is_float() && t1.size() == t2.size())
    } else {
        assert!(t1.is_scalar() && t2.is_scalar() && t1.size() == t2.size())
    }

    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = *t;
    Ok(match t {
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => Box::new(move |ctx| {
            let lhs = lhs.execute(ctx) as i64;
            let rhs = rhs.execute(ctx) as i64;

            flag_policy.sub_carry(t, lhs as u64, rhs as u64, ctx);

            lhs.overflowing_sub(rhs).0 as u64 & t.gen_mask()
        }),
        Type::F32 | Type::F64 => unreachable!("invalid type: {:?}", t),
        Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_lshl(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = *t;
    Ok(match t {
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => {
            Box::new(move |ctx| (lhs.execute(ctx) << rhs.execute(ctx)) & t.gen_mask() as u64)
        }
        Type::F32 | Type::F64 | Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_lshr(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
        Type::F32 | Type::F64 | Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_ashr(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
        Type::F32 | Type::F64 | Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_rotr(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
        Type::F32 | Type::F64 | Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_load(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
        Type::Void => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_zext_cast(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
        Type::F32 | Type::F64 | Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_sext_cast(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    let from = op.get_type();
    let to = t.gen_mask();
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
        Type::F32 | Type::F64 | Type::Void | Type::Bool => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_bit_cast(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    let op = compile_op(op, flag_policy.clone())?;
    let to = t.gen_mask();

    Ok(Box::new(move |ctx| unsafe { op.execute(ctx) & to }))
}

unsafe fn gen_and(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
        | Type::I64 => Box::new(move |ctx| (lhs.execute(ctx) & rhs.execute(ctx)) & t.gen_mask()),
        Type::F32 | Type::F64 | Type::Void => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_or(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
        | Type::I64 => Box::new(move |ctx| (lhs.execute(ctx) | rhs.execute(ctx)) & t.gen_mask()),
        Type::F32 | Type::F64 | Type::Void => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_xor(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
        | Type::I64 => Box::new(move |ctx| (lhs.execute(ctx) ^ rhs.execute(ctx)) & t.gen_mask()),
        Type::F32 | Type::F64 | Type::Void => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_not(
    t: &Type,
    op: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
        | Type::I64 => Box::new(move |ctx| (!op.execute(ctx) & t.gen_mask())),
        Type::F32 | Type::F64 | Type::Void => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_if(
    t: &Type,
    cond: &Operand,
    if_true: &Operand,
    if_false: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
    assert_eq!(cond.get_type(), Type::Bool);

    let cond = compile_op(cond, flag_policy.clone())?;
    let if_true = compile_op(if_true, flag_policy.clone())?;
    let if_false = compile_op(if_false, flag_policy.clone())?;

    let t = *t;
    Ok(match t {
        _ => Box::new(move |ctx| {
            t.gen_mask()
                & if cond.execute(ctx) != 0 {
                    if_true.execute(ctx)
                } else {
                    if_false.execute(ctx)
                }
        }),
    })
}

unsafe fn gen_cmp_eq(
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
            (lhs.execute(ctx) & t.gen_mask() == rhs.execute(ctx) & t.gen_mask()) as u64
        }),
        Type::Void => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_cmp_ne(
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
            (lhs.execute(ctx) & t.gen_mask() != rhs.execute(ctx) & t.gen_mask()) as u64
        }),
        Type::Void => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_cmp_gt(
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
            (lhs.execute(ctx) & t.gen_mask() > rhs.execute(ctx) & t.gen_mask()) as u64
        }),
        Type::Void => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_cmp_lt(
    op1: &Operand,
    op2: &Operand,
    flag_policy: Arc<dyn FlagPolicy>,
) -> Result<Box<dyn CompiledCode>, CodegenError> {
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
            (lhs.execute(ctx) & t.gen_mask() < rhs.execute(ctx) & t.gen_mask()) as u64
        }),
        Type::Void => unreachable!("invalid type: {:?}", t),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::codegen::flag_policy::DummyFlagPolicy;
    use crate::interrupt::NoModel;
    use crate::loader::elf::ElfLoader;
    use crate::vm_builder::VmBuilder;

    #[test]
    fn test_compile_simple_imm_ir() {
        let loader = ElfLoader::new(&[]);
        let mut vm = VmBuilder::new(loader).build(0);

        let flag_policy = Arc::new(DummyFlagPolicy) as Arc<dyn FlagPolicy>;

        //Test UADD
        let ir = Ir::Add(
            Type::U64,
            Operand::Immediate(Type::U64, 30),
            Operand::Immediate(Type::U64, 50),
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
            Operand::Immediate(Type::I64, (-10i64) as u64),
            Operand::Immediate(Type::I64, 10),
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
            Operand::Immediate(Type::U8, 10),
            Operand::Immediate(Type::U8, 9),
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
            Operand::Immediate(Type::I16, 10),
            Operand::Immediate(Type::I16, (-10i16) as u64),
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
            Operand::Immediate(Type::I32, 10),
            Operand::Immediate(Type::I32, (-10i64) as u64),
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
            Operand::Immediate(Type::F64, (10f64).to_bits()),
            Operand::Immediate(Type::F64, (4.5f64).to_bits()),
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
            Operand::Immediate(Type::I64, 10_u64),
            Operand::Immediate(Type::I64, (-5i64) as u64),
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
            Operand::Immediate(Type::F64, (10f64).to_bits()),
            Operand::Immediate(Type::F64, (3.3f64).to_bits()),
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
            Operand::Immediate(Type::U8, 0b1111_1111),
            Operand::Immediate(Type::U8, 4),
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
            Operand::Immediate(Type::U8, 0b0000_1111),
            Operand::Immediate(Type::U8, 4),
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
            Operand::Immediate(Type::U8, 0b1010_1111),
            Operand::Immediate(Type::U8, 2),
        );
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result, 0b1110_1011);

        //Test Zext
        let ir = Ir::ZextCast(Type::U16, Operand::Immediate(Type::U16, (-10i8) as u64));
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result as u16, 65526);

        //Test Sext
        let ir = Ir::SextCast(Type::I64, Operand::Immediate(Type::I8, 0xFF));
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result as i64, -1);

        //Test Sext-32
        let ir = Ir::SextCast(Type::I32, Operand::Immediate(Type::I8, 0xFF));
        let result = unsafe {
            compile_ir(&ir, flag_policy.clone())
                .unwrap()
                .execute(&mut vm)
        };
        assert_eq!(result as i64, u32::max_value() as i64);
    }
}
