#![feature(generators, generator_trait)]
#![feature(impl_trait_in_assoc_type)]
#![allow(unreachable_code)]
pub mod codegen;
mod soft_mmu;
use core::{
    ir::{BasicBlock, BasicBlockTerminator},
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

        let mut current_mem = [0u8; 4096];
        let mut current_off = 0;

        self.mmu.read_all_at(current_off, &mut current_mem);
        loop {
            // Process device IRQs
            if let Some(irq) = self.irq_queue.recv() {
                todo!()
            }

            // Try to decode and compile instructions into BasicBlock
            let mut bb = BasicBlock::new(current_off);
            let mut inst_offs = 0u64;
            loop {
                let Some(raw_inst) = A::Inst::decode(&current_mem[inst_offs as usize..])
                else {
                    // If failed to parse an instruction, we need to read a new memory
                    current_off += inst_offs;
                    inst_offs = 0;

                    self.mmu.read_all_at(current_off, &mut current_mem);
                    continue;
                };

                raw_inst.compile_to_ir(&mut bb);
                inst_offs += raw_inst.size();
                if bb.terminator() != BasicBlockTerminator::None {
                    // If we have a terminator, we can stop parsing instructions
                    break;
                }
            }

            current_off += inst_offs;
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

            // prepare next address
            match bb.terminator() {
                BasicBlockTerminator::None => unreachable!("BasicBlock should have a terminator"),
                BasicBlockTerminator::Next => (),
                BasicBlockTerminator::BranchCond { cond, target } => todo!(),
                BasicBlockTerminator::Branch(_) => todo!(),
            };
        }

        unreachable!()
    }
}
