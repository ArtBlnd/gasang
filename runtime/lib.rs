#![feature(generators, generator_trait)]
#![feature(impl_trait_in_assoc_type)]
pub mod abi;
pub mod codegen;
mod soft_mmu;
use core::{
    ir::{BasicBlock, BasicBlockTerminator, IrValue, IrType},
    Architecture, Instruction, Interrupt, ArchitectureCompat, Register,
};
use std::{
    collections::BinaryHeap,
    convert::Infallible,
    ops::{Generator, GeneratorState},
    pin::pin,
};

use abi::Abi;
use codegen::{Codegen, Executable};
use device::{IoDevice, IrqQueue};
pub use soft_mmu::*;

use crate::codegen::Context;

pub struct Runtime;
impl Runtime {
    pub unsafe fn run<A, C, I, F>(binary: &[u8], prepare: F) -> Infallible
    where
        A: Architecture,
        C: ArchitectureCompat<A> + Codegen,
        I: ArchitectureCompat<A> + Abi,
        F: FnOnce(&mut SoftMmu, &mut IrqQueue),
    {
        let mut mmu = SoftMmu::new();
        let mut irq = IrqQueue::new();
        prepare(&mut mmu, &mut irq);

        let mut abi = I::new();
        let cgn = C::new();
        let ctx = C::allocate_execution_context::<A>();

        // Initializes the ABI, execution context, and mmu with given binary
        abi.on_initialize(binary, &ctx, &mmu);

        let mut curr_mem = [0u8; 4096];
        let mut next_off = ctx.get(IrValue::Register(IrType::U64, A::get_pc_register().raw()));

        mmu.read_all_at(next_off, &mut curr_mem);
        loop {
            // Process device IRQs
            let mut irq_queue = BinaryHeap::new();
            while let Some(irq) = irq.recv() {
                irq_queue.push(irq);
            }

            for irq in irq_queue {
                abi.on_irq(irq.id, irq.level, &ctx, &mmu);
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

                    mmu.read_all_at(next_off, &mut curr_mem);
                    continue;
                };

                raw_inst.compile_to_ir(&mut bb);
                total_inst_size += raw_inst.size();
                if bb.terminator() != BasicBlockTerminator::None {
                    // If we have a terminator, we can stop parsing instructions
                    break;
                }
            }

            let compiled_bb = cgn.compile(&bb);
            let gen = compiled_bb.execute(&ctx, &mmu);
            let mut gen = pin!(gen);

            while let GeneratorState::Yielded(interrupt) = gen.as_mut().resume(()) {
                match interrupt {
                    Interrupt::Exception(id) => abi.on_exception(id, &ctx, &mmu),
                    Interrupt::Interrupt(id) => abi.on_interrupt(id, &ctx, &mmu),
                    Interrupt::SystemCall(id) => abi.on_system_call(id, &ctx, &mmu),
                    Interrupt::Aborts(value) => std::process::exit(value),
                    Interrupt::Reset => std::process::exit(0),
                    Interrupt::Yield => std::thread::yield_now(),
                    Interrupt::WaitForInterrupt => {
                        // TODO: wait for a interrupt using Parking
                        loop {
                            let Some(irq) = irq.recv() 
                            else {
                                std::thread::yield_now();
                                continue;
                            };
                            
                            abi.on_irq(irq.id, irq.level, &ctx, &mmu);
                        }
                    },
                    Interrupt::DivideByZero => {
                        panic!("divide by zero");
                    }
                }
            }

            // prepare next address
            match bb.terminator() {
                BasicBlockTerminator::None => unreachable!("BasicBlock should have a terminator"),
                BasicBlockTerminator::Next => next_off += total_inst_size,
                BasicBlockTerminator::BranchCond { cond, target } => {
                    let cond: u64 = ctx.get(cond);
                    let dest: u64 = ctx.get(target);

                    if cond != 0 {
                        next_off = dest;
                    } else {
                        next_off += total_inst_size;
                    }
                }
                BasicBlockTerminator::Branch(target) => {
                    next_off = ctx.get(target);
                }
            };
        }
    }
}
