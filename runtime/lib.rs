#![feature(generators, generator_trait)]
#![feature(impl_trait_in_assoc_type)]
#![allow(unreachable_code)]
pub mod codegen;
mod soft_mmu;
use core::{
    ir::{BasicBlock, BasicBlockTerminator, IrConstant},
    Architecture, Instruction, Interrupt,
};
use std::{
    convert::Infallible,
    ops::{Generator, GeneratorState},
    pin::pin,
};

use codegen::{rustjit::RustjitCodegen, Codegen, Executable};
use device::{IoDevice, IrqQueue};
pub use soft_mmu::*;

use crate::codegen::Context;

pub struct Runtime {
    mmu: SoftMmu,
    irq_queue: IrqQueue,
}

impl Runtime {
    pub unsafe fn run<A, F>(&mut self, prepare: F) -> Infallible
    where
        A: Architecture,
        F: FnOnce(&mut SoftMmu, &mut IrqQueue),
    {
        // prepare the runtime
        prepare(&mut self.mmu, &mut self.irq_queue);
        let compiler = RustjitCodegen;
        let mut ctx = RustjitCodegen::new_context::<A>();

        let mut curr_mem = [0u8; 4096];
        let mut next_off = 0;

        self.mmu.read_all_at(next_off, &mut curr_mem);
        loop {
            // Process device IRQs
            if let Some(irq) = self.irq_queue.recv() {
                todo!()
            }

            // Try to decode and compile instructions into BasicBlock
            let mut bb = BasicBlock::new(next_off);
            let mut total_inst_size = 0u64;
            loop {
                let Some(raw_inst) = A::Inst::decode(&curr_mem[total_inst_size as usize..])
                else {
                    // If failed to parse an instruction, we need to read a new memory
                    next_off += total_inst_size;
                    total_inst_size = 0;

                    self.mmu.read_all_at(next_off, &mut curr_mem);
                    continue;
                };

                raw_inst.compile_to_ir(&mut bb);
                total_inst_size += raw_inst.size();
                if bb.terminator() != BasicBlockTerminator::None {
                    // If we have a terminator, we can stop parsing instructions
                    break;
                }
            }

            {
                let compiled_bb = compiler.compile(&bb);
                let gen = compiled_bb.execute(&mut ctx, &self.mmu);
                let mut gen = pin!(gen);

                while let GeneratorState::Yielded(interrupt) = gen.as_mut().resume(()) {
                    match interrupt {
                        Interrupt::SystemCall(id) => (),
                        Interrupt::Yield => std::thread::yield_now(),
                        _ => (),
                    }
                }
            }

            // prepare next address
            match bb.terminator() {
                BasicBlockTerminator::None => unreachable!("BasicBlock should have a terminator"),
                BasicBlockTerminator::Next => next_off += total_inst_size,
                BasicBlockTerminator::BranchCond { cond, target } => {
                    let IrConstant::U64(cond) = ctx.evaluate(cond)
                    else {
                        unreachable!("Condition should be a u64");
                    };
                    let IrConstant::U64(jump_to) = ctx.evaluate(target)
                    else {
                        unreachable!("Jump target should be a u64");
                    };

                    if cond != 0 {
                        next_off = jump_to;
                    } else {
                        next_off += total_inst_size;
                    }
                }
                BasicBlockTerminator::Branch(target) => {
                    let IrConstant::U64(jump_to) = ctx.evaluate(target)
                    else {
                        unreachable!("Jump target should be a u64");
                    };

                    next_off = jump_to;
                }
            };
        }

        unreachable!()
    }
}
