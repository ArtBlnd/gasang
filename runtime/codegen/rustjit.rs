pub mod context;
mod register_file;
use arch_desc::aarch64::AArch64Architecture;
use num_traits::{PrimInt, WrappingAdd, WrappingMul, WrappingSub};
pub use register_file::*;

use core::{
    ir::{BasicBlock, Flag, IrInst, IrType, IrValue, TypeOf},
    Architecture, ArchitectureCompat, Interrupt,
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    ops::Generator,
};

use crate::SoftMmu;

use self::context::RustjitContext;
use super::{
    analysis::{Analysis, VariableLivenessAnalysis},
    Codegen, Context, Executable,
};

pub struct RustjitExectuable {
    exec: Vec<Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>>>,
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

    fn compile(&self, bb: &BasicBlock) -> Self::Executable {
        let mut exec: Vec<Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>>> = Vec::new();

        let variable_liveness = VariableLivenessAnalysis::new(bb).analyze();
        let max_variables = variable_liveness.maximum_variable_live();

        let mut var_allocation_map = HashMap::new();
        let mut var_allocation_ids: VecDeque<_> = (0usize..max_variables).collect();

        let mut allocate_variable = |value: IrValue, idx: usize| -> IrValue {
            let IrValue::Variable(ty, id) = value
            else {
                return value;
            };

            let allocated_id = var_allocation_map
                .entry(id)
                .or_insert(var_allocation_ids.pop_front().unwrap())
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
                    let dst = allocate_variable(dst, idx);
                    let lhs = allocate_variable(lhs, idx);
                    let rhs = allocate_variable(rhs, idx);

                    gen_add(dst, lhs, rhs)
                }
                &IrInst::Sub { dst, lhs, rhs } => {
                    let dst = allocate_variable(dst, idx);
                    let lhs = allocate_variable(lhs, idx);
                    let rhs = allocate_variable(rhs, idx);

                    gen_sub(dst, lhs, rhs)
                }
                &IrInst::Mul { dst, lhs, rhs } => {
                    let dst = allocate_variable(dst, idx);
                    let lhs = allocate_variable(lhs, idx);
                    let rhs = allocate_variable(rhs, idx);

                    gen_mul(dst, lhs, rhs)
                }
                &IrInst::Div { dst, lhs, rhs } => {
                    let dst = allocate_variable(dst, idx);
                    let lhs = allocate_variable(lhs, idx);
                    let rhs = allocate_variable(rhs, idx);

                    gen_div(dst, lhs, rhs)
                }
                &IrInst::MoveFlag { dst, dst_pos, flag } => {
                    let dst = allocate_variable(dst, idx);

                    gen_move_flag(dst, dst_pos, flag)
                }
                _ => todo!(),
            };

            exec.push(inst);
        }

        // free variable space
        exec.push(Box::new(move |ctx: &RustjitContext, _: &SoftMmu| {
            *ctx.variables.borrow_mut() = None;
            None
        }) as Box<_>);

        RustjitExectuable { exec }
    }
}

fn gen_add(
    dst: IrValue,
    lhs: IrValue,
    rhs: IrValue,
) -> Box<dyn Fn(&RustjitContext, &SoftMmu) -> Option<Interrupt>> {
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
        IrType::U8 => gen_add_impl!(u8),
        IrType::U16 => gen_add_impl!(u16),
        IrType::U32 => gen_add_impl!(u32),
        IrType::U64 => gen_add_impl!(u64),
        IrType::U128 => gen_add_impl!(u128),
        IrType::I8 => gen_add_impl!(i8),
        IrType::I16 => gen_add_impl!(i16),
        IrType::I32 => gen_add_impl!(i32),
        IrType::I64 => gen_add_impl!(i64),
        IrType::I128 => gen_add_impl!(i128),

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
        IrType::U8 => gen_sub_impl!(u8),
        IrType::U16 => gen_sub_impl!(u16),
        IrType::U32 => gen_sub_impl!(u32),
        IrType::U64 => gen_sub_impl!(u64),
        IrType::U128 => gen_sub_impl!(u128),
        IrType::I8 => gen_sub_impl!(i8),
        IrType::I16 => gen_sub_impl!(i16),
        IrType::I32 => gen_sub_impl!(i32),
        IrType::I64 => gen_sub_impl!(i64),
        IrType::I128 => gen_sub_impl!(i128),

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
        IrType::U8 => gen_mul_impl!(u8),
        IrType::U16 => gen_mul_impl!(u16),
        IrType::U32 => gen_mul_impl!(u32),
        IrType::U64 => gen_mul_impl!(u64),
        IrType::U128 => gen_mul_impl!(u128),
        IrType::I8 => gen_mul_impl!(i8),
        IrType::I16 => gen_mul_impl!(i16),
        IrType::I32 => gen_mul_impl!(i32),
        IrType::I64 => gen_mul_impl!(i64),
        IrType::I128 => gen_mul_impl!(i128),

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
                    return Some(Interrupt::DivideByZero);
                }

                let v = lhs.wrapping_div(rhs);
                ctx.set::<$ty>(dst, v);
                ctx.set_flag(Flag::ZF, v == 0);

                None
            }) as Box<_>
        };
    }

    match dst.ty() {
        IrType::U8 => gen_div_impl!(u8),
        IrType::U16 => gen_div_impl!(u16),
        IrType::U32 => gen_div_impl!(u32),
        IrType::U64 => gen_div_impl!(u64),
        IrType::U128 => gen_div_impl!(u128),
        IrType::I8 => gen_div_impl!(i8),
        IrType::I16 => gen_div_impl!(i16),
        IrType::I32 => gen_div_impl!(i32),
        IrType::I64 => gen_div_impl!(i64),
        IrType::I128 => gen_div_impl!(i128),

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
