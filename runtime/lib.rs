#![feature(generators, generator_trait)]
#![feature(impl_trait_in_assoc_type)]

pub mod codegen;
mod soft_mmu;
use device::IrqQueue;
pub use soft_mmu::*;

pub struct Runtime {
    mmu: SoftMmu,
    irq_queue: IrqQueue,
}
