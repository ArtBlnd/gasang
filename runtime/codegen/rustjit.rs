mod register_file;
mod value;

use core::{
    ir::{BasicBlock, IrInst, IrValue},
    Architecture, Interrupt,
};
use std::{collections::HashMap, ops::Generator};

use device::devices::SoftMmu;
pub use register_file::*;
use value::RustjitValue;

use super::{Codegen, Executable};
pub struct RustjitContext {
    registers: RegisterFile,
    variables: HashMap<usize, RustjitValue>,
}

impl RustjitContext {
    fn get(&self, value: IrValue) -> RustjitValue {
        match value {
            IrValue::Variable(_ty, id) => *self.variables.get(&id).unwrap(),
            IrValue::Register(ty, id) => self.registers.get_value(id, ty),
            IrValue::Constant(constant) => constant.into(),
        }
    }

    fn set(&mut self, value: IrValue, data: RustjitValue) {
        match value {
            IrValue::Variable(_, id) => {
                self.variables.insert(id, data);
            }
            IrValue::Register(_, id) => {
                self.registers.set_value(id, &data);
            }
            IrValue::Constant(_) => panic!("Constant cannot be set"),
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
        io_device: &'a SoftMmu,
    ) -> Self::Generator<'a> {
        || {
            for inst in &self.inst {
                let Some(interrput) = inst(context, io_device) else {
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

    fn compile(&self, bb: BasicBlock) -> Self::Executable {
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
