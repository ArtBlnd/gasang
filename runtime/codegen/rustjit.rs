pub mod context;
mod register_file;
use arch_desc::aarch64::AArch64Architecture;
use num_traits::{PrimInt, WrappingAdd, WrappingMul, WrappingSub};
pub use register_file::*;
use smallvec::SmallVec;

use core::{
    ir::{BasicBlock, BasicBlockTerminator, Flag, IrInst, IrType, IrValue, TypeOf},
    Architecture, ArchitectureCompat, Interrupt, Register,
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    mem,
    ops::Generator,
};

use crate::IoDevice;
use crate::SoftMmu;

use self::context::RustjitContext;
use super::{
    analysis::{Analysis, VariableLivenessAnalysis},
    Codegen, Context, Executable,
};

pub struct RustjitExectuable {
    exec: Vec<Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>>>,
    terminator: Box<dyn Fn(&RustjitContext, &SoftMmu)>,
}

impl Executable for RustjitExectuable {
    type Context = RustjitContext;
    type Generator<'a> = impl Generator<Yield = Interrupt, Return = ()> + 'a;

    unsafe fn execute<'a>(
        &'a self,
        context: &'a Self::Context,
        mmu: &'a SoftMmu,
    ) -> Self::Generator<'a> {
        || {
            for inst in &self.exec {
                let Some(interrput) = inst(context, mmu) else {
                    continue;
                };

                yield interrput;
            }

            (self.terminator)(context, mmu);
        }
    }
}

pub struct RustjitCodegen;
impl ArchitectureCompat<AArch64Architecture> for RustjitCodegen {}

impl Codegen for RustjitCodegen {
    type Context = RustjitContext;
    type Executable = RustjitExectuable;

    fn new() -> Self {
        Self
    }

    fn allocate_execution_context<A: Architecture>() -> Self::Context {
        let mut flag = Vec::new();
        flag.resize_with(Flag::max(), || Cell::new(false));

        RustjitContext {
            registers: RegisterFile::new(&A::get_register_file_desc()),
            variables: RefCell::new(None),
            flag: flag.into_boxed_slice(),
        }
    }

    fn compile<A: Architecture>(&self, bb: &BasicBlock) -> Self::Executable {
        let mut exec: Vec<Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>>> = Vec::new();

        let variable_liveness = VariableLivenessAnalysis::new(bb).analyze();
        let max_variables = variable_liveness.maximum_variable_live();

        let mut var_allocation_map = HashMap::new();
        let mut var_allocation_ids: VecDeque<_> = (0usize..max_variables).collect();

        let mut map_variable = |value: IrValue, idx: usize| -> IrValue {
            let IrValue::Variable(ty, id) = value
            else {
                return value;
            };

            let allocated_id = var_allocation_map
                .entry(id)
                .or_insert_with(|| var_allocation_ids.pop_front().unwrap())
                .clone();

            if variable_liveness.is_dead_after(idx, &value) {
                var_allocation_ids.push_back(allocated_id);
                var_allocation_map.remove(&id);
            }

            IrValue::Variable(ty, allocated_id)
        };

        // initialize variable space
        exec.push(Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
            *ctx.variables.borrow_mut() =
                Some(vec![Cell::new(0); max_variables].into_boxed_slice());
            None
        }) as Box<_>);

        for (idx, inst) in bb.inst().iter().enumerate() {
            let inst = match inst {
                &IrInst::Add { dst, lhs, rhs } => {
                    let dst = map_variable(dst, idx);
                    let lhs = map_variable(lhs, idx);
                    let rhs = map_variable(rhs, idx);

                    gen_add(dst, lhs, rhs)
                }
                &IrInst::Sub { dst, lhs, rhs } => {
                    let dst = map_variable(dst, idx);
                    let lhs = map_variable(lhs, idx);
                    let rhs = map_variable(rhs, idx);

                    gen_sub(dst, lhs, rhs)
                }
                &IrInst::Mul { dst, lhs, rhs } => {
                    let dst = map_variable(dst, idx);
                    let lhs = map_variable(lhs, idx);
                    let rhs = map_variable(rhs, idx);

                    gen_mul(dst, lhs, rhs)
                }
                &IrInst::Div { dst, lhs, rhs } => {
                    let dst = map_variable(dst, idx);
                    let lhs = map_variable(lhs, idx);
                    let rhs = map_variable(rhs, idx);

                    gen_div(dst, lhs, rhs)
                }
                &IrInst::Rem { dst, lhs, rhs } => {
                    let dst = map_variable(dst, idx);
                    let lhs = map_variable(lhs, idx);
                    let rhs = map_variable(rhs, idx);

                    gen_rem(dst, lhs, rhs)
                }
                &IrInst::And { dst, lhs, rhs } => {
                    let dst = map_variable(dst, idx);
                    let lhs = map_variable(lhs, idx);
                    let rhs = map_variable(rhs, idx);

                    gen_bit_and(dst, lhs, rhs)
                }
                &IrInst::Or { dst, lhs, rhs } => {
                    let dst = map_variable(dst, idx);
                    let lhs = map_variable(lhs, idx);
                    let rhs = map_variable(rhs, idx);

                    gen_bit_or(dst, lhs, rhs)
                }
                &IrInst::Xor { dst, lhs, rhs } => {
                    let dst = map_variable(dst, idx);
                    let lhs = map_variable(lhs, idx);
                    let rhs = map_variable(rhs, idx);

                    gen_bit_xor(dst, lhs, rhs)
                }
                &IrInst::Not { dst, src } => {
                    let dst = map_variable(dst, idx);
                    let src = map_variable(src, idx);

                    gen_bit_not(dst, src)
                }
                &IrInst::MoveFlag { dst, dst_pos, flag } => {
                    let dst = map_variable(dst, idx);

                    gen_move_flag(dst, dst_pos, flag)
                }
                &IrInst::Assign { dst, src } => {
                    let src = map_variable(src, idx);
                    let dst = map_variable(dst, idx);

                    gen_assign(dst, src)
                }
                &IrInst::Shl { dst, lhs, rhs } => {
                    let lhs = map_variable(lhs, idx);
                    let rhs = map_variable(rhs, idx);
                    let dst = map_variable(dst, idx);

                    gen_shl(dst, lhs, rhs)
                }
                &IrInst::Load { dst, src } => {
                    let src = map_variable(src, idx);
                    let dst = map_variable(dst, idx);

                    gen_load(dst, src)
                }
                &IrInst::ZextCast { dst, src } => {
                    let src = map_variable(src, idx);
                    let dst = map_variable(dst, idx);

                    gen_zext_cast(dst, src)
                }
                x => todo!("todo {:?}", x),
            };

            exec.push(inst);
        }

        let terminator = match bb.terminator() {
            BasicBlockTerminator::None => unreachable!("unreachable basic block"),
            BasicBlockTerminator::Next => Box::new(|_: &RustjitContext, _: &SoftMmu| {}) as Box<_>,
            BasicBlockTerminator::BranchCond {
                cond,
                target_true,
                target_false,
            } => {
                let cond = map_variable(cond, bb.inst().len());
                Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                    let cond = ctx.get::<u64>(cond);

                    let pc = IrValue::Register(IrType::B64, A::get_pc_register().raw());
                    if cond != 0 {
                        ctx.set(pc, ctx.get::<u64>(target_true));
                    } else {
                        ctx.set(pc, ctx.get::<u64>(target_false));
                    }
                }) as Box<_>
            }
            BasicBlockTerminator::Branch(target) => {
                let target = map_variable(target, bb.inst().len());
                Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                    let pc = IrValue::Register(IrType::B64, A::get_pc_register().raw());
                    ctx.set(pc, ctx.get::<u64>(target));
                }) as Box<_>
            }
        };

        RustjitExectuable { exec, terminator }
    }
}

fn gen_add(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    #[inline(always)]
    fn carrying_add<T: WrappingAdd + PrimInt>(a: T, b: T, carry_in: bool) -> (T, bool, bool, bool) {
        let carry = if carry_in { T::one() } else { T::zero() };
        let sum = a.wrapping_add(&b).wrapping_add(&carry);
        let cf = (a > sum) || (carry == T::one() && b == T::max_value());
        let of = (a ^ sum).leading_zeros() == 0;
        let zf = sum.is_zero();
        (sum, cf, of, zf)
    }

    macro_rules! gen_add_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                let (v, cf, of, zf) = carrying_add(lhs, rhs, false);
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::CF, cf);
                ctx.set_flag(Flag::OF, of);
                ctx.set_flag(Flag::ZF, zf);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_add_impl!(u8),
        IrType::B16 => gen_add_impl!(u16),
        IrType::B32 => gen_add_impl!(u32),
        IrType::B64 => gen_add_impl!(u64),
        IrType::B128 => gen_add_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_sub(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    #[inline(always)]
    fn carrying_sub<T: WrappingSub + PrimInt>(a: T, b: T, carry_in: bool) -> (T, bool, bool, bool) {
        let carry = if carry_in { T::one() } else { T::zero() };
        let sum = a.wrapping_sub(&b).wrapping_sub(&carry);
        let cf = (a < sum) || (carry == T::one() && b == T::max_value());
        let of = (a ^ sum).leading_zeros() == 0;
        let zf = sum.is_zero();
        (sum, cf, of, zf)
    }

    macro_rules! gen_sub_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                let (v, cf, of, zf) = carrying_sub(lhs, rhs, false);
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::CF, cf);
                ctx.set_flag(Flag::OF, of);
                ctx.set_flag(Flag::ZF, zf);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_sub_impl!(u8),
        IrType::B16 => gen_sub_impl!(u16),
        IrType::B32 => gen_sub_impl!(u32),
        IrType::B64 => gen_sub_impl!(u64),
        IrType::B128 => gen_sub_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_mul(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    #[inline(always)]
    fn carrying_mul<T: WrappingMul + PrimInt>(a: T, b: T, carry_in: bool) -> (T, bool, bool, bool) {
        let carry = if carry_in { T::one() } else { T::zero() };
        let sum = a.wrapping_mul(&b).wrapping_mul(&carry);
        let cf = (a > sum) || (carry == T::one() && b == T::max_value());
        let of = (a ^ sum).leading_zeros() == 0;
        let zf = sum.is_zero();
        (sum, cf, of, zf)
    }

    macro_rules! gen_mul_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                let (v, cf, of, zf) = carrying_mul(lhs, rhs, false);
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::CF, cf);
                ctx.set_flag(Flag::OF, of);
                ctx.set_flag(Flag::ZF, zf);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_mul_impl!(u8),
        IrType::B16 => gen_mul_impl!(u16),
        IrType::B32 => gen_mul_impl!(u32),
        IrType::B64 => gen_mul_impl!(u64),
        IrType::B128 => gen_mul_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_div(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    macro_rules! gen_div_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                if rhs == 0 {
                    return Some(Interrupt::Exception(0));
                }

                let v = lhs.wrapping_div(rhs);
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_div_impl!(u8),
        IrType::B16 => gen_div_impl!(u16),
        IrType::B32 => gen_div_impl!(u32),
        IrType::B64 => gen_div_impl!(u64),
        IrType::B128 => gen_div_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_rem(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    macro_rules! gen_rem_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                if rhs == 0 {
                    return Some(Interrupt::Exception(0));
                }

                let v = lhs.wrapping_rem(rhs);
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_rem_impl!(u8),
        IrType::B16 => gen_rem_impl!(u16),
        IrType::B32 => gen_rem_impl!(u32),
        IrType::B64 => gen_rem_impl!(u64),
        IrType::B128 => gen_rem_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_bit_and(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    macro_rules! gen_bit_and_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                let v = lhs & rhs;
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_bit_and_impl!(u8),
        IrType::B16 => gen_bit_and_impl!(u16),
        IrType::B32 => gen_bit_and_impl!(u32),
        IrType::B64 => gen_bit_and_impl!(u64),
        IrType::B128 => gen_bit_and_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_bit_or(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    macro_rules! gen_bit_or_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                let v = lhs | rhs;
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_bit_or_impl!(u8),
        IrType::B16 => gen_bit_or_impl!(u16),
        IrType::B32 => gen_bit_or_impl!(u32),
        IrType::B64 => gen_bit_or_impl!(u64),
        IrType::B128 => gen_bit_or_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_bit_xor(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    macro_rules! gen_bit_xor_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                let v = lhs ^ rhs;
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_bit_xor_impl!(u8),
        IrType::B16 => gen_bit_xor_impl!(u16),
        IrType::B32 => gen_bit_xor_impl!(u32),
        IrType::B64 => gen_bit_xor_impl!(u64),
        IrType::B128 => gen_bit_xor_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_bit_not(
    dst: IrValue,
    src: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == src.ty());
    macro_rules! gen_bit_not_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let src: $ty = ctx.get(src);

                let v = !src;
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_bit_not_impl!(u8),
        IrType::B16 => gen_bit_not_impl!(u16),
        IrType::B32 => gen_bit_not_impl!(u32),
        IrType::B64 => gen_bit_not_impl!(u64),
        IrType::B128 => gen_bit_not_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_shl(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty());
    macro_rules! gen_shl_impl {
        ($lhs_ty:ty, $rhs_ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $lhs_ty = ctx.get(lhs);
                let rhs: $rhs_ty = ctx.get(rhs);

                let v = lhs << rhs;
                ctx.set::<$lhs_ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match (lhs.ty(), rhs.ty()) {
        (IrType::B8, IrType::B8) => gen_shl_impl!(u8, u8),
        (IrType::B16, IrType::B8) => gen_shl_impl!(u16, u8),
        (IrType::B32, IrType::B8) => gen_shl_impl!(u32, u8),
        (IrType::B64, IrType::B8) => gen_shl_impl!(u64, u8),
        (IrType::B128, IrType::B8) => gen_shl_impl!(u128, u8),

        _ => unimplemented!("Unsupported type: {:?} << {:?}", lhs.ty(), rhs.ty()),
    }
}

fn gen_lshr(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    macro_rules! gen_lshr_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                let v = lhs >> rhs;
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_lshr_impl!(u8),
        IrType::B16 => gen_lshr_impl!(u16),
        IrType::B32 => gen_lshr_impl!(u32),
        IrType::B64 => gen_lshr_impl!(u64),
        IrType::B128 => gen_lshr_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_ashr(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == lhs.ty() && lhs.ty() == rhs.ty());
    macro_rules! gen_ashr_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let lhs: $ty = ctx.get(lhs);
                let rhs: $ty = ctx.get(rhs);

                let v = lhs >> rhs;
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_ashr_impl!(u8),
        IrType::B16 => gen_ashr_impl!(u16),
        IrType::B32 => gen_ashr_impl!(u32),
        IrType::B64 => gen_ashr_impl!(u64),
        IrType::B128 => gen_ashr_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_move_flag(
    dst: IrValue,
    dst_pos: usize,
    flag: Flag,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
        let v = ctx.get_flag(flag) as u64;
        ctx.set::<u64>(dst, v << dst_pos as u64);
        None
    })
}

fn gen_assign(
    dst: IrValue,
    src: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty() == src.ty());
    macro_rules! gen_assign_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let src: $ty = ctx.get(src);

                ctx.set::<$ty>(dst, src);
                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_assign_impl!(u8),
        IrType::B16 => gen_assign_impl!(u16),
        IrType::B32 => gen_assign_impl!(u32),
        IrType::B64 => gen_assign_impl!(u64),
        IrType::B128 => gen_assign_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_load(
    dst: IrValue,
    src: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    // TODO: check that size of src is same as pointer size
    macro_rules! gen_load_impl {
        ($src_ty:ty, $dst_ty:ty) => {
            Box::new(move |ctx: &RustjitContext, mmu: &SoftMmu| unsafe {
                let src: $src_ty = ctx.get(src);
                let src = src as u64;

                let mut buf = SmallVec::<[u8; 32]>::new();
                buf.resize(mem::size_of::<$dst_ty>(), 0);
                mmu.read_at(src, &mut buf);
                ctx.set::<$dst_ty>(
                    dst,
                    <$dst_ty>::from_ne_bytes(buf[..mem::size_of::<$dst_ty>()].try_into().unwrap()),
                );
                None
            }) as Box<_>
        };
    }

    match (src.ty(), dst.ty()) {
        (IrType::B64, IrType::B8) => gen_load_impl!(u64, u8),
        (IrType::B64, IrType::B16) => gen_load_impl!(u64, u16),
        (IrType::B64, IrType::B32) => gen_load_impl!(u64, u32),
        (IrType::B64, IrType::B64) => gen_load_impl!(u64, u64),
        (IrType::B64, IrType::B128) => gen_load_impl!(u64, u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}

fn gen_zext_cast(
    dst: IrValue,
    src: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
    assert!(dst.ty().size_of() >= src.ty().size_of());
    macro_rules! gen_zext_cast_impl {
        ($ty:ty) => {
            Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
                let src_ext = match src.ty() {
                    IrType::B8 => ctx.get::<u8>(src) as $ty,
                    IrType::B16 => ctx.get::<u16>(src) as $ty,
                    IrType::B32 => ctx.get::<u32>(src) as $ty,
                    IrType::B64 => ctx.get::<u64>(src) as $ty,
                    IrType::B128 => ctx.get::<u128>(src) as $ty,

                    _ => unimplemented!("Unsupported type: {:?}", src.ty()),
                };

                ctx.set::<$ty>(dst, src_ext);
                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::B8 => gen_zext_cast_impl!(u8),
        IrType::B16 => gen_zext_cast_impl!(u16),
        IrType::B32 => gen_zext_cast_impl!(u32),
        IrType::B64 => gen_zext_cast_impl!(u64),
        IrType::B128 => gen_zext_cast_impl!(u128),

        _ => unimplemented!("Unsupported type: {:?}", dst.ty()),
    }
}
