mod register_file;
use core::{
    ir::{BasicBlock, IrInst, IrValue},
    Architecture, Interrupt,
};
use std::{collections::HashMap, ops::Generator};

pub use register_file::*;

use crate::SoftMmu;

use super::{Codegen, Executable};
pub struct RustjitContext {
    registers: RegisterFile,
    variables: HashMap<usize, u64>,
}

impl RustjitContext {
    pub fn get(&self, value: IrValue) -> u64 {
        match value {
            IrValue::Variable(_ty, _id) => todo!(),
            IrValue::Register(_ty, _id) => todo!(),
            IrValue::Constant(_constant) => todo!(),
        }
    }
    pub fn set(&mut self, value: IrValue, _data: u64) {
        match value {
            IrValue::Variable(_ty, _id) => todo!(),
            IrValue::Register(_ty, _id) => todo!(),
            IrValue::Constant(_constant) => todo!(),
        }
    }
}

pub struct RustjitExectuable {
    inst: Vec<Box<dyn Fn(&mut RustjitContext, &SoftMmu) -> Option<Interrupt>>>,
}

impl Executable for RustjitExectuable {
    type Context = RustjitContext;
    type Generator<'a> = impl Generator<Yield = Interrupt, Return = ()> + 'a;

    unsafe fn execute<'a>(
        &'a self,
        context: &'a mut Self::Context,
        mmu: &'a SoftMmu,
    ) -> Self::Generator<'a> {
        || {
            for inst in &self.inst {
                let Some(interrput) = inst(context, mmu) else {
                    continue;
                };

                yield interrput;
            }
        }
    }
}

pub struct RustjitCodegen;

impl Codegen for RustjitCodegen {
    type Context = RustjitContext;
    type Executable = RustjitExectuable;

    fn new_context<A: Architecture>() -> Self::Context {
        RustjitContext {
            registers: RegisterFile::new(&A::get_register_file_desc()),
            variables: HashMap::new(),
        }
    }

    fn compile(&self, bb: &BasicBlock) -> Self::Executable {
        let mut executable = RustjitExectuable { inst: Vec::new() };

        for inst in bb.inst() {
            let inst = match inst {
                &IrInst::Add { dst, lhs, rhs } => {
                    Box::new(move |ctx: &mut RustjitContext, _: &SoftMmu| {
                        let lhs = ctx.get(lhs);
                        let rhs = ctx.get(rhs);
                        ctx.set(dst, lhs + rhs);

                        None
                    }) as Box<_>
                }
                &IrInst::Sub { dst, lhs, rhs } => {
                    Box::new(move |ctx: &mut RustjitContext, _: &SoftMmu| {
                        let lhs = ctx.get(lhs);
                        let rhs = ctx.get(rhs);
                        ctx.set(dst, lhs - rhs);

                        None
                    }) as Box<_>
                }
                _ => todo!(),
            };

            executable.inst.push(inst);
        }

        executable
    }
}
