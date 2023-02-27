use smallvec::SmallVec;

use crate::codegen::flag_policy::{DummyFlagPolicy, FlagPolicy};
use crate::codegen::*;
use crate::error::CodegenError;
use crate::ir::{BlockDestination, Ir, Operand, Type, VecType};
use crate::value::Value;

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
    type Exec = FnExec<Value>;
    type ExecBlock = FnExec<()>;

    fn compile_ir(&self, ir: &Ir) -> Self::Exec {
        unsafe { compile_ir(&ir, self.flag_policy.clone()).unwrap() }
    }

    fn compile_ir_block(&self, ir_block: &IrBlock) -> Self::ExecBlock {
        let ip_inc = ir_block.original_size() as u64;
        let compile_results: SmallVec<[(FnExec<Value>, BlockDestination); 2]> = ir_block
            .items()
            .iter()
            .map(|item| (self.compile_ir(item.root()), item.dest().clone()))
            .collect();

        FnExec::new(move |ctx| unsafe { execute(&compile_results, ctx, ip_inc) })
    }
}

unsafe fn execute(
    code: &SmallVec<[(FnExec<Value>, BlockDestination); 2]>,
    ctx: &mut ExecutionContext,
    ip_inc: u64,
) {
    let mut ip_modified = false;
    for (exec, dest) in code {
        let val = unsafe { exec.execute(ctx) };
        handle_block_dest(dest.clone(), val, ctx, &mut ip_modified);
    }

    if !ip_modified {
        ctx.cpu.set_pc(ctx.cpu.pc() + ip_inc);
    }
}

unsafe fn handle_block_dest<'a>(
    dest: BlockDestination,
    val: Value,
    ctx: &mut ExecutionContext<'a>,
    ip_modified: &mut bool,
) {
    match dest {
        BlockDestination::Flags => {
            ctx.cpu.set_flag(val.u64());
        }
        BlockDestination::Pc => {
            ctx.cpu.set_pc(val.u64());
            *ip_modified = true;
        }
        BlockDestination::None => { /* do nothing */ }
        BlockDestination::Gpr(ty, reg_id) => {
            let gpr = ctx.cpu.gpr_mut(reg_id);

            match ty {
                Type::U8 | Type::I8 => *gpr.u8_mut() = val.u8(),
                Type::U16 | Type::I16 => *gpr.u16_mut() = val.u16(),
                Type::U32 | Type::I32 => *gpr.u32_mut() = val.u32(),
                Type::U64 | Type::I64 => *gpr.u64_mut() = val.u64(),
                _ => unreachable!(),
            }
        }
        BlockDestination::Fpr(ty, reg_id) => {
            let fpr = ctx.cpu.fpr_mut(reg_id);

            match ty {
                Type::U8 | Type::I8 => *fpr.u8_mut() = val.u8(),
                Type::U16 | Type::I16 => *fpr.u16_mut() = val.u16(),
                Type::U32 | Type::I32 => *fpr.u32_mut() = val.u32(),
                Type::U64 | Type::I64 => *fpr.u64_mut() = val.u64(),
                Type::F32 => *fpr.f32_mut() = val.f32(),
                Type::F64 => *fpr.f64_mut() = val.f64(),
                Type::Vec(VecType::U64, 2) => *fpr.u64x2_mut() = val.u64x2(),
                _ => unreachable!(),
            }
        }
        BlockDestination::Sys(ty, reg_id) => {
            let sys = ctx.cpu.sys_mut(reg_id);

            match ty {
                Type::U8 | Type::I8 => *sys.u8_mut() = val.u8(),
                Type::U16 | Type::I16 => *sys.u16_mut() = val.u16(),
                Type::U32 | Type::I32 => *sys.u32_mut() = val.u32(),
                Type::U64 | Type::I64 => *sys.u64_mut() = val.u64(),
                _ => unreachable!(),
            }
        }
        BlockDestination::FprSlot(ty, reg_id, slot_id) => {
            let fpr = ctx.cpu.fpr_mut(reg_id);

            match ty {
                Type::U8 | Type::I8 => fpr.u8_slice_mut()[slot_id as usize] = val.u8(),
                Type::U16 | Type::I16 => fpr.u16_slice_mut()[slot_id as usize] = val.u16(),
                Type::U32 | Type::I32 => fpr.u32_slice_mut()[slot_id as usize] = val.u32(),
                Type::U64 | Type::I64 => fpr.u64_slice_mut()[slot_id as usize] = val.u64(),
                _ => unreachable!(),
            }
        }
        BlockDestination::Exit => {
            panic!("Exit");
        }
        BlockDestination::Memory(ty, addr) => {
            match ty {
                Type::U8 | Type::I8 => ctx.mmu.frame(addr).write_u8(val.u8()),
                Type::U16 | Type::I16 => ctx.mmu.frame(addr).write_u16(val.u16()),
                Type::U32 | Type::I32 | Type::F32 => ctx.mmu.frame(addr).write_u32(val.u32()),
                Type::U64 | Type::I64 | Type::F64 => ctx.mmu.frame(addr).write_u64(val.u64()),
                Type::Vec(VecType::U64 | VecType::I64, 2) => {
                    ctx.mmu.frame(addr).write(&val.u8_slice_ref()[..16])
                }
                _ => unreachable!(),
            }
            .unwrap();
        }
        BlockDestination::MemoryRelI64(ty, reg_id, offs) => {
            let (addr, of) = ctx.cpu.gpr(reg_id).u64().overflowing_add_signed(offs);
            assert_eq!(of, false);

            match ty {
                Type::U8 | Type::I8 => ctx.mmu.frame(addr).write_u8(val.u8()),
                Type::U16 | Type::I16 => ctx.mmu.frame(addr).write_u16(val.u16()),
                Type::U32 | Type::I32 | Type::F32 => ctx.mmu.frame(addr).write_u32(val.u32()),
                Type::U64 | Type::I64 | Type::F64 => ctx.mmu.frame(addr).write_u64(val.u64()),
                Type::Vec(VecType::U64 | VecType::I64, 2) => {
                    ctx.mmu.frame(addr).write(&val.u8_slice_ref()[..16])
                }
                _ => unreachable!(),
            }
            .unwrap();
        }
        BlockDestination::MemoryRelU64(ty, reg_id, offs) => {
            let (addr, of) = ctx.cpu.gpr(reg_id).u64().overflowing_add(offs);
            assert_eq!(of, false);

            match ty {
                Type::U8 | Type::I8 => ctx.mmu.frame(addr).write_u8(val.u8()),
                Type::U16 | Type::I16 => ctx.mmu.frame(addr).write_u16(val.u16()),
                Type::U32 | Type::I32 | Type::F32 => ctx.mmu.frame(addr).write_u32(val.u32()),
                Type::U64 | Type::I64 | Type::F64 => ctx.mmu.frame(addr).write_u64(val.u64()),
                Type::Vec(VecType::U64 | VecType::I64, 2) => {
                    ctx.mmu.frame(addr).write(&val.u8_slice_ref()[..16])
                }
                _ => unreachable!(),
            }
            .unwrap();
        }
        BlockDestination::MemoryIr(ir) => {
            let ty = ir.get_type();
            let addr = compile_ir(&ir, DummyFlagPolicy).unwrap().execute(ctx).u64();

            match ty {
                Type::U8 | Type::I8 => ctx.mmu.frame(addr).write_u8(val.u8()),
                Type::U16 | Type::I16 => ctx.mmu.frame(addr).write_u16(val.u16()),
                Type::U32 | Type::I32 | Type::F32 => ctx.mmu.frame(addr).write_u32(val.u32()),
                Type::U64 | Type::I64 | Type::F64 => ctx.mmu.frame(addr).write_u64(val.u64()),
                Type::Vec(VecType::U64 | VecType::I64, 2) => {
                    ctx.mmu.frame(addr).write(&val.u8_slice_ref()[..16])
                }
                _ => unreachable!(),
            }
            .unwrap();
        }
    }
}

unsafe fn compile_ir<T>(ir: &Ir, flag_policy: T) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    // Optimiations
    match ir {
        Ir::ZextCast(_, Operand::Ir(ir)) => {
            return Ok(compile_ir(ir, flag_policy.clone())?);
        }
        Ir::Add(Type::U64, Operand::Ip, Operand::Immediate(Type::I64, imm)) => {
            let imm = *imm;
            return Ok(FnExec::new(move |ctx| {
                (ctx.cpu.pc() as i64 + imm as i64).into()
            }));
        }
        Ir::Add(Type::U64, Operand::Ip, Operand::Immediate(Type::U64, imm)) => {
            let imm = *imm;
            return Ok(FnExec::new(move |ctx| (ctx.cpu.pc() + imm).into()));
        }
        Ir::Value(Operand::Ir(ir)) => return compile_ir(ir, flag_policy.clone()),
        Ir::Value(Operand::Immediate(t, imm)) => {
            let imm = *imm;
            let _t = *t;

            return Ok(FnExec::new(move |_ctx| imm.into()));
        }
        Ir::Value(Operand::Gpr(t, reg)) => {
            let reg = *reg;
            let t = *t;
            return Ok(FnExec::new(move |ctx| match t {
                Type::U8 | Type::I8 => (ctx.cpu.gpr(reg).u8()).into(),
                Type::U16 | Type::I16 => (ctx.cpu.gpr(reg).u16()).into(),
                Type::U32 | Type::I32 => (ctx.cpu.gpr(reg).u32()).into(),
                Type::U64 | Type::I64 => (ctx.cpu.gpr(reg).u64()).into(),
                _ => unreachable!("Invalid type"),
            }));
        }
        Ir::LShr(t, op1, Operand::Immediate(t1, val)) => {
            assert!(t == t1, "Type mismatch");
            let op1 = compile_op(op1, flag_policy.clone())?;
            let val = *val;
            let t = *t;

            return Ok(FnExec::new(move |ctx| {
                let mut ret = op1.execute(ctx);
                match t.size() {
                    1 => *ret.u8_mut() >>= val,
                    2 => *ret.u16_mut() >>= val,
                    4 => *ret.u32_mut() >>= val,
                    8 => *ret.u64_mut() >>= val,
                    _ => unreachable!(),
                }

                ret
            }));
        }
        Ir::And(Type::U64, Operand::Flag, Operand::Immediate(Type::U64, imm)) => {
            let imm = *imm;
            return Ok(FnExec::new(move |ctx| (ctx.cpu.flag() & imm).into()));
        }
        Ir::And(t, Operand::Immediate(t1, imm1), Operand::Immediate(t2, imm2)) => {
            assert!(t1 == t2 && t1 == t, "Type mismatch");
            let imm1 = *imm1;
            let imm2 = *imm2;
            let _t = *t;

            return Ok(FnExec::new(move |_ctx| (imm1 & imm2).into()));
        }
        Ir::If(t, Operand::Immediate(t1, imm), op1, op2) => {
            assert!(*t1 == Type::Bool, "Type mismatch");
            assert!(op1.get_type() == op2.get_type(), "Type mismatch");
            let imm = *imm;
            let _t = *t;
            let op1 = compile_op(op1, flag_policy.clone())?;
            let op2 = compile_op(op2, flag_policy.clone())?;

            return Ok(FnExec::new(move |ctx| {
                if imm != 0 {
                    op1.execute(ctx)
                } else {
                    op2.execute(ctx)
                }
            }));
        }

        _ => {}
    };

    match ir {
        Ir::Add(t, op1, op2) => gen_add(t, op1, op2, flag_policy),
        Ir::Sub(t, op1, op2) => gen_sub(t, op1, op2, flag_policy),
        Ir::Mul(t, op1, op2) => gen_mul(t, op1, op2, flag_policy),
        Ir::Div(t, op1, op2) => gen_div(t, op1, op2, flag_policy),
        Ir::Mod(t, op1, op2) => gen_mod(t, op1, op2, flag_policy),
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
        Ir::Nop => Ok(FnExec::new(|_| Value::new(0))),

        Ir::If(t, cond, if_true, if_false) => gen_if(t, cond, if_true, if_false, flag_policy),
        Ir::CmpEq(op1, op2) => gen_cmp_eq(op1, op2, flag_policy),
        Ir::CmpNe(op1, op2) => gen_cmp_ne(op1, op2, flag_policy),
        Ir::CmpGt(op1, op2) => gen_cmp_gt(op1, op2, flag_policy),
        Ir::CmpLt(op1, op2) => gen_cmp_lt(op1, op2, flag_policy),
    }
}

unsafe fn compile_op<T>(op: &Operand, flag_policy: T) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    Ok(match op {
        Operand::Ir(ir) => compile_ir(ir, flag_policy.clone())?,
        Operand::Gpr(t, reg) => {
            let reg = *reg;
            let t = *t;
            FnExec::new(move |ctx| match t {
                Type::U8 | Type::I8 => Value::from_u8(ctx.cpu.gpr(reg).u8()),
                Type::U16 | Type::I16 => Value::from_u16(ctx.cpu.gpr(reg).u16()),
                Type::U32 | Type::I32 => Value::from_u32(ctx.cpu.gpr(reg).u32()),
                Type::U64 | Type::I64 => Value::from_u64(ctx.cpu.gpr(reg).u64()),
                _ => unreachable!("Invalid type"),
            })
        }
        Operand::Fpr(t, reg) => {
            let reg = *reg;
            let t = *t;
            FnExec::new(move |ctx| match t {
                Type::U8 | Type::I8 => Value::from_u8(ctx.cpu.fpr(reg).u8()),
                Type::U16 | Type::I16 => Value::from_u16(ctx.cpu.fpr(reg).u16()),
                Type::U32 | Type::I32 => Value::from_u32(ctx.cpu.fpr(reg).u32()),
                Type::U64 | Type::I64 => Value::from_u64(ctx.cpu.fpr(reg).u64()),
                Type::Vec(VecType::U64 | VecType::I64, 2) => {
                    let mut ret = Value::new(16);
                    *ret.u64x2_mut() = ctx.cpu.fpr(reg).u64x2();
                    ret
                }
                _ => unreachable!("Invalid type"),
            })
        }
        Operand::Sys(t, reg) => {
            let reg = *reg;
            let t = *t;
            FnExec::new(move |ctx| match t {
                Type::U8 | Type::I8 => Value::from_u8(ctx.cpu.sys(reg).u8()),
                Type::U16 | Type::I16 => Value::from_u16(ctx.cpu.sys(reg).u16()),
                Type::U32 | Type::I32 => Value::from_u32(ctx.cpu.sys(reg).u32()),
                Type::U64 | Type::I64 => Value::from_u64(ctx.cpu.sys(reg).u64()),
                _ => unreachable!("Invalid type"),
            })
        }
        Operand::Immediate(t, imm) => {
            let imm = *imm & t.gen_mask();
            let t = *t;
            FnExec::new(move |_| match t {
                Type::U8 | Type::I8 => Value::from_u8(imm as u8),
                Type::U16 | Type::I16 => Value::from_u16(imm as u16),
                Type::U32 | Type::I32 => Value::from_u32(imm as u32),
                Type::U64 | Type::I64 => Value::from_u64(imm),
                _ => unreachable!("Invalid type"),
            })
        }
        Operand::ImmediateValue(t, imm) => {
            let imm = imm.clone();
            let t = *t;
            FnExec::new(move |_| match t {
                Type::U8 | Type::I8 => Value::from_u8(imm.u8()),
                Type::U16 | Type::I16 => Value::from_u16(imm.u16()),
                Type::U32 | Type::I32 => Value::from_u32(imm.u32()),
                Type::U64 | Type::I64 => Value::from_u64(imm.u64()),
                Type::Vec(VecType::U64, 2) => {
                    let mut ret = Value::new(16);
                    *ret.u64x2_mut() = imm.u64x2();
                    ret
                }
                _ => unreachable!("Invalid type"),
            })
        }
        Operand::VoidIr(ir) => {
            let ir = compile_ir(ir, flag_policy.clone())?;
            FnExec::new(move |ctx| {
                ir.execute(ctx);
                Value::new(0)
            })
        }
        Operand::Ip => FnExec::new(move |ctx| Value::from_u64(ctx.cpu.pc())),
        Operand::Flag => FnExec::new(move |ctx| Value::from_u64(ctx.cpu.flag())),
        Operand::Dbg(s, op) => {
            let op = compile_op(op, flag_policy.clone())?;
            let s = s.to_string();
            FnExec::new(move |ctx| {
                let val = op.execute(ctx);
                println!("{s}, {:x}", val.u64());
                val
            })
        }
    })
}

unsafe fn gen_add<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
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
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u8();
            let rhs = rhs.execute(ctx).u8();

            lhs.overflowing_add(rhs).0.into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u16();
            let rhs = rhs.execute(ctx).u16();

            lhs.overflowing_add(rhs).0.into()
        }),
        Type::U32 | Type::I32 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u32();
            let rhs = rhs.execute(ctx).u32();

            lhs.overflowing_add(rhs).0.into()
        }),
        Type::U64 | Type::I64 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u64();
            let rhs = rhs.execute(ctx).u64();

            lhs.overflowing_add(rhs).0.into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_sub<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
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
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u8();
            let rhs = rhs.execute(ctx).u8();

            lhs.overflowing_sub(rhs).0.into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u16();
            let rhs = rhs.execute(ctx).u16();

            lhs.overflowing_sub(rhs).0.into()
        }),
        Type::U32 | Type::I32 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u32();
            let rhs = rhs.execute(ctx).u32();

            lhs.overflowing_sub(rhs).0.into()
        }),
        Type::U64 | Type::I64 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u64();
            let rhs = rhs.execute(ctx).u64();

            lhs.overflowing_sub(rhs).0.into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_mul<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u8();
            let rhs = rhs.execute(ctx).u8();

            lhs.overflowing_mul(rhs).0.into()
        }),
        Type::U16 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u16();
            let rhs = rhs.execute(ctx).u16();

            lhs.overflowing_mul(rhs).0.into()
        }),
        Type::U32 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u32();
            let rhs = rhs.execute(ctx).u32();

            lhs.overflowing_mul(rhs).0.into()
        }),
        Type::U64 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u64();
            let rhs = rhs.execute(ctx).u64();

            lhs.overflowing_mul(rhs).0.into()
        }),
        Type::I8 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).i8();
            let rhs = rhs.execute(ctx).i8();

            lhs.overflowing_mul(rhs).0.into()
        }),
        Type::I16 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).i16();
            let rhs = rhs.execute(ctx).i16();

            lhs.overflowing_mul(rhs).0.into()
        }),
        Type::I32 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).i32();
            let rhs = rhs.execute(ctx).i32();

            lhs.overflowing_mul(rhs).0.into()
        }),
        Type::I64 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).i64();
            let rhs = rhs.execute(ctx).i64();

            lhs.overflowing_mul(rhs).0.into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_div<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u8();
            let rhs = rhs.execute(ctx).u8();

            lhs.overflowing_div(rhs).0.into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u16();
            let rhs = rhs.execute(ctx).u16();

            lhs.overflowing_div(rhs).0.into()
        }),
        Type::U32 | Type::I32 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u32();
            let rhs = rhs.execute(ctx).u32();

            lhs.overflowing_div(rhs).0.into()
        }),
        Type::U64 | Type::I64 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u64();
            let rhs = rhs.execute(ctx).u64();

            lhs.overflowing_div(rhs).0.into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_mod<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u8();
            let rhs = rhs.execute(ctx).u8();

            (lhs % rhs).into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u16();
            let rhs = rhs.execute(ctx).u16();

            (lhs % rhs).into()
        }),
        Type::U32 | Type::I32 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u32();
            let rhs = rhs.execute(ctx).u32();

            (lhs % rhs).into()
        }),
        Type::U64 | Type::I64 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u64();
            let rhs = rhs.execute(ctx).u64();

            (lhs % rhs).into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_addc<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
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
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u8();
            let rhs = rhs.execute(ctx).u8();

            flag_policy.add_carry(t, lhs as u64, rhs as u64, ctx.cpu);
            lhs.overflowing_add(rhs).0.into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u16();
            let rhs = rhs.execute(ctx).u16();

            flag_policy.add_carry(t, lhs as u64, rhs as u64, ctx.cpu);
            lhs.overflowing_add(rhs).0.into()
        }),
        Type::U32 | Type::I32 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u32();
            let rhs = rhs.execute(ctx).u32();

            flag_policy.add_carry(t, lhs as u64, rhs as u64, ctx.cpu);
            lhs.overflowing_add(rhs).0.into()
        }),
        Type::U64 | Type::I64 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u64();
            let rhs = rhs.execute(ctx).u64();

            flag_policy.add_carry(t, lhs, rhs, ctx.cpu);
            lhs.overflowing_add(rhs).0.into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_subc<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
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
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u8();
            let rhs = rhs.execute(ctx).u8();

            flag_policy.sub_carry(t, lhs as u64, rhs as u64, ctx.cpu);
            lhs.overflowing_sub(rhs).0.into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u16();
            let rhs = rhs.execute(ctx).u16();

            flag_policy.sub_carry(t, lhs as u64, rhs as u64, ctx.cpu);
            lhs.overflowing_sub(rhs).0.into()
        }),
        Type::U32 | Type::I32 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u32();
            let rhs = rhs.execute(ctx).u32();

            flag_policy.sub_carry(t, lhs as u64, rhs as u64, ctx.cpu);
            lhs.overflowing_sub(rhs).0.into()
        }),
        Type::U64 | Type::I64 => FnExec::new(move |ctx| {
            let lhs = lhs.execute(ctx).u64();
            let rhs = rhs.execute(ctx).u64();

            flag_policy.sub_carry(t, lhs, rhs, ctx.cpu);
            lhs.overflowing_sub(rhs).0.into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_lshl<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = *t;
    Ok(match t {
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u8_mut() <<= rhs.u8();
            lhs
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u16_mut() <<= rhs.u16();
            lhs
        }),
        Type::U32 | Type::I32 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u32_mut() <<= rhs.u32();
            lhs
        }),
        Type::U64 | Type::I64 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u64_mut() <<= rhs.u64();
            lhs
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_lshr<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u8_mut() >>= rhs.u8();
            lhs
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u16_mut() >>= rhs.u16();
            lhs
        }),
        Type::U32 | Type::I32 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u32_mut() >>= rhs.u32();
            lhs
        }),
        Type::U64 | Type::I64 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u64_mut() >>= rhs.u64();
            lhs
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_ashr<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    assert!(op2.get_type().is_scalar());
    assert!(op2.get_type().is_unsigned());

    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u8_mut() >>= rhs.u8();
            lhs
        }),
        Type::U16 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u16_mut() >>= rhs.u16();
            lhs
        }),
        Type::U32 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u32_mut() >>= rhs.u32();
            lhs
        }),
        Type::U64 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.u64_mut() >>= rhs.u64();
            lhs
        }),
        Type::I8 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.i8_mut() >>= rhs.u8();
            lhs
        }),
        Type::I16 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.i16_mut() >>= rhs.u16();
            lhs
        }),
        Type::I32 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.i32_mut() >>= rhs.u32();
            lhs
        }),
        Type::I64 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            *lhs.i64_mut() >>= rhs.u64();
            lhs
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_rotr<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    assert!(op2.get_type().is_scalar());
    assert!(op2.get_type().is_unsigned());

    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    Ok(match t {
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            lhs.u8_mut().rotate_right(rhs.u8() as u32).into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            lhs.u16_mut().rotate_right(rhs.u16() as u32).into()
        }),
        Type::U32 | Type::I32 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            lhs.u32_mut().rotate_right(rhs.u32()).into()
        }),
        Type::U64 | Type::I64 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            lhs.u64_mut().rotate_right(rhs.u64() as u32).into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_load<T>(t: &Type, op: &Operand, flag_policy: T) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let op = compile_op(op, flag_policy.clone())?;
    Ok(match t {
        Type::Bool => FnExec::new(move |ctx| {
            let mut var = op.execute(ctx);
            (ctx.mmu.frame(*var.u64_mut()).read_u8().unwrap() & 0b1).into()
        }),
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let mut var = op.execute(ctx);
            ctx.mmu.frame(*var.u64_mut()).read_u8().unwrap().into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let mut var = op.execute(ctx);
            ctx.mmu.frame(*var.u64_mut()).read_u16().unwrap().into()
        }),
        Type::U32 | Type::I32 | Type::F32 => FnExec::new(move |ctx| {
            let mut var = op.execute(ctx);
            ctx.mmu.frame(*var.u64_mut()).read_u32().unwrap().into()
        }),
        Type::U64 | Type::I64 | Type::F64 => FnExec::new(move |ctx| {
            let mut var = op.execute(ctx);
            ctx.mmu.frame(*var.u64_mut()).read_u64().unwrap().into()
        }),
        Type::Vec(VecType::U64, 2) => FnExec::new(move |ctx| {
            let mut var = op.execute(ctx);
            let mut mem = ctx.mmu.frame(*var.u64_mut());

            let mut value = Value::new(16);
            mem.read(value.u8_slice_mut()).unwrap();

            value
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_zext_cast<T>(
    t: &Type,
    op: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let op = compile_op(op, flag_policy.clone())?;
    Ok(match t {
        Type::U8
        | Type::U16
        | Type::U32
        | Type::U64
        | Type::I8
        | Type::I16
        | Type::I32
        | Type::I64 => FnExec::new(move |ctx| op.execute(ctx)),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_sext_cast<T>(
    t: &Type,
    op: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let from = op.get_type();
    let to = t.gen_mask() as i64;
    let op = compile_op(op, flag_policy.clone())?;

    Ok(match from {
        Type::U8 | Type::U16 | Type::U32 | Type::U64 => op,
        Type::I8 => FnExec::new(move |ctx| {
            let v: i64 = (*op.execute(ctx).i8_mut()).into();
            (v & to).into()
        }),
        Type::I16 => FnExec::new(move |ctx| {
            let v: i64 = (*op.execute(ctx).i16_mut()).into();
            (v & to).into()
        }),
        Type::I32 => FnExec::new(move |ctx| {
            let v: i64 = (*op.execute(ctx).i32_mut()).into();
            (v & to).into()
        }),
        Type::I64 => FnExec::new(move |ctx| {
            let v: i64 = *op.execute(ctx).i64_mut();
            (v & to).into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_bit_cast<T>(
    t: &Type,
    op: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let op = compile_op(op, flag_policy.clone())?;
    let t = *t;

    Ok(FnExec::new(move |ctx| unsafe {
        op.execute(ctx).truncate_to(t)
    }))
}

unsafe fn gen_and<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
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
        | Type::I64 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let mut rhs = rhs.execute(ctx);

            *lhs.u64_mut() &= *rhs.u64_mut();
            lhs
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_or<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
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
        | Type::I64 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let mut rhs = rhs.execute(ctx);

            *lhs.u64_mut() |= *rhs.u64_mut();
            lhs
        }),
        Type::Vec(VecType::U64, 2) => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let rhs = rhs.execute(ctx);

            lhs.u64x2_mut()[0] |= rhs.u64x2()[0];
            lhs.u64x2_mut()[1] |= rhs.u64x2()[1];

            lhs
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_xor<T>(
    t: &Type,
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
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
        | Type::I64 => FnExec::new(move |ctx| {
            let mut lhs = lhs.execute(ctx);
            let mut rhs = rhs.execute(ctx);

            *lhs.u64_mut() ^= *rhs.u64_mut();
            lhs
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_not<T>(t: &Type, op: &Operand, flag_policy: T) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
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
        | Type::I64 => {
            FnExec::new(move |ctx| ((!*op.execute(ctx).u64_mut()) & t.gen_mask()).into())
        }
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_if<T>(
    t: &Type,
    cond: &Operand,
    if_true: &Operand,
    if_false: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    assert_eq!(cond.get_type(), Type::Bool);

    let cond = compile_op(cond, flag_policy.clone())?;
    let if_true = compile_op(if_true, flag_policy.clone())?;
    let if_false = compile_op(if_false, flag_policy.clone())?;

    let _t = *t;
    Ok(FnExec::new(move |ctx| {
        if *cond.execute(ctx).u64_mut() != 0 {
            if_true.execute(ctx)
        } else {
            if_false.execute(ctx)
        }
    }))
}

unsafe fn gen_cmp_eq<T>(
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = op1.get_type();
    assert_eq!(op1.get_type(), op2.get_type());

    Ok(match t {
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u8_mut();
            let rhs = *rhs.execute(ctx).u8_mut();

            (lhs == rhs).into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u16_mut();
            let rhs = *rhs.execute(ctx).u16_mut();

            (lhs == rhs).into()
        }),
        Type::U32 | Type::I32 | Type::F32 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u32_mut();
            let rhs = *rhs.execute(ctx).u32_mut();

            (lhs == rhs).into()
        }),
        Type::U64 | Type::I64 | Type::F64 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u64_mut();
            let rhs = *rhs.execute(ctx).u64_mut();

            (lhs == rhs).into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_cmp_ne<T>(
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = op1.get_type();
    assert_eq!(op1.get_type(), op2.get_type());

    Ok(match t {
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u8_mut();
            let rhs = *rhs.execute(ctx).u8_mut();

            (lhs != rhs).into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u16_mut();
            let rhs = *rhs.execute(ctx).u16_mut();

            (lhs != rhs).into()
        }),
        Type::U32 | Type::I32 | Type::F32 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u32_mut();
            let rhs = *rhs.execute(ctx).u32_mut();

            (lhs != rhs).into()
        }),
        Type::U64 | Type::I64 | Type::F64 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u64_mut();
            let rhs = *rhs.execute(ctx).u64_mut();

            (lhs != rhs).into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_cmp_gt<T>(
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = op1.get_type();
    assert_eq!(op1.get_type(), op2.get_type());

    Ok(match t {
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u8_mut();
            let rhs = *rhs.execute(ctx).u8_mut();

            (lhs > rhs).into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u16_mut();
            let rhs = *rhs.execute(ctx).u16_mut();

            (lhs > rhs).into()
        }),
        Type::U32 | Type::I32 | Type::F32 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u32_mut();
            let rhs = *rhs.execute(ctx).u32_mut();

            (lhs > rhs).into()
        }),
        Type::U64 | Type::I64 | Type::F64 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u64_mut();
            let rhs = *rhs.execute(ctx).u64_mut();

            (lhs > rhs).into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}

unsafe fn gen_cmp_lt<T>(
    op1: &Operand,
    op2: &Operand,
    flag_policy: T,
) -> Result<FnExec<Value>, CodegenError>
where
    T: FlagPolicy + Clone + 'static,
{
    let lhs = compile_op(op1, flag_policy.clone())?;
    let rhs = compile_op(op2, flag_policy.clone())?;

    let t = op1.get_type();
    assert_eq!(op1.get_type(), op2.get_type());

    Ok(match t {
        Type::U8 | Type::I8 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u8_mut();
            let rhs = *rhs.execute(ctx).u8_mut();

            (lhs < rhs).into()
        }),
        Type::U16 | Type::I16 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u16_mut();
            let rhs = *rhs.execute(ctx).u16_mut();

            (lhs < rhs).into()
        }),
        Type::U32 | Type::I32 | Type::F32 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u32_mut();
            let rhs = *rhs.execute(ctx).u32_mut();

            (lhs < rhs).into()
        }),
        Type::U64 | Type::I64 | Type::F64 => FnExec::new(move |ctx| {
            let lhs = *lhs.execute(ctx).u64_mut();
            let rhs = *rhs.execute(ctx).u64_mut();

            (lhs < rhs).into()
        }),
        _ => unreachable!("invalid type: {:?}", t),
    })
}
