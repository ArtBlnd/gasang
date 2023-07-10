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
    pub unsafe fn run<A, C, I>(binary: &[u8], prepare: impl FnOnce(&mut SoftMmu, &mut IrqQueue)) -> Infallible
    where
        A: Architecture,
        C: ArchitectureCompat<A> + Codegen,
        I: ArchitectureCompat<A> + Abi,
    {
        let pc_reg = IrValue::Register(IrType::B64, A::get_pc_register().raw());

        let mut mmu = SoftMmu::new();
        let mut irq = IrqQueue::new();
        prepare(&mut mmu, &mut irq);

        let mut abi = I::new();
        let mut ctx = C::allocate_execution_context::<A>();
        let cgn = C::new();

        // Initializes the ABI, execution context, and mmu with given binary
        abi.on_initialize(binary, &mut ctx, &mut mmu);

        let mut buffer = [0u8; 4096];
        loop {
            let mut pc = ctx.get(pc_reg);
            mmu.read_all_at(pc, &mut buffer);

            // Process device IRQs
            let mut irq_queue = BinaryHeap::new();
            while let Some(irq) = irq.recv() {
                irq_queue.push(irq);
            }

            for irq in irq_queue {
                abi.on_irq(irq.id, irq.level, &ctx, &mmu);
            }

            // Try to decode and compile instructions into BasicBlock
            let mut bb = BasicBlock::new(pc);
            let mut total_inst_size = 0u64;
            loop {
                let Some(raw_inst) = A::Inst::decode(&buffer[total_inst_size as usize..])
                else {
                    // If failed to parse an instruction, we need to read a new memory
                    pc += total_inst_size;
                    total_inst_size = 0;

                    mmu.read_all_at(pc, &mut buffer);
                    if total_inst_size == 0 {
                        panic!("Failed to decode instruction at 0x{:x}", pc);
                    }
                    break;
                };

                raw_inst.compile_to_ir(&mut bb);
                total_inst_size += raw_inst.size();
                if bb.terminator() != BasicBlockTerminator::None {
                    // If we have a terminator, we can stop parsing instructions
                    break;
                }
            }

            let compiled_bb = cgn.compile::<A>(&bb);
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
                }
            }
        }
    }
}
