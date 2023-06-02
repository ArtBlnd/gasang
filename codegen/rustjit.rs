mod register_file;
use core::{
    ir::{BasicBlock, IrInst, IrValue},
    Architecture,
};
use std::collections::HashMap;

pub use register_file::*;

use crate::{Codegen, Executable};

pub struct RustjitContext {
    registers: RegisterFile,
    variables: HashMap<usize, u64>,
}

impl RustjitContext {
    pub fn get(&self, value: IrValue) -> u64 {
        match value {
            IrValue::Variable(ty, id) => todo!(),
            IrValue::Register(ty, id) => todo!(),
            IrValue::Constant(constant) => todo!(),
        }
    }
    pub fn set(&mut self, value: IrValue, data: u64) {
        match value {
            IrValue::Variable(ty, id) => todo!(),
            IrValue::Register(ty, id) => todo!(),
            IrValue::Constant(constant) => todo!(),
        }
    }
}

pub struct RustjitExectuable {
    executables: Vec<Box<dyn Fn(&mut RustjitContext)>>,
}

impl Executable for RustjitExectuable {
    type Context = RustjitContext;

    unsafe fn execute(&self, context: &mut Self::Context) {
        for executable in &self.executables {
            executable(context);
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

    fn compile(&self, bb: BasicBlock) -> Self::Executable {
        let mut executable = RustjitExectuable {
            executables: Vec::new(),
        };

        for inst in bb.inst() {
            let inst = match inst {
                &IrInst::Add { dst, lhs, rhs } => Box::new(move |ctx: &mut RustjitContext| {
                    let lhs = ctx.get(lhs);
                    let rhs = ctx.get(rhs);
                    ctx.set(dst, lhs + rhs);
                }) as Box<_>,
                &IrInst::Sub { dst, lhs, rhs } => Box::new(move |ctx: &mut RustjitContext| {
                    let lhs = ctx.get(lhs);
                    let rhs = ctx.get(rhs);
                    ctx.set(dst, lhs - rhs);
                }) as Box<_>,
                _ => todo!(),
            };

            executable.executables.push(inst);
        }

        executable
    }
}
