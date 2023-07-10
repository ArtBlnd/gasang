use crate::{codegen::Context, SoftMmu};

mod aarch64_unknown_linux;
pub use aarch64_unknown_linux::*;
use device::IoDevice;

pub trait Abi {
    fn new() -> Self;

    /// Initialize the ABI with the given binary.
    fn on_initialize<C: Context>(&mut self, binary: &[u8], ctx: &mut C, mmu: &mut SoftMmu);
    /// Called when an exception occurs.
    fn on_exception<C: Context>(&self, exception: u64, ctx: &C, mmu: &SoftMmu);
    /// Called when an interrupt occurs.
    fn on_interrupt<C: Context>(&self, interrupt: u64, ctx: &C, mmu: &SoftMmu);
    /// Called when a system call occurs.
    fn on_system_call<C: Context>(&self, system_call: u64, ctx: &C, mmu: &SoftMmu);
    /// Called when an IRQ occurs.
    fn on_irq<C: Context>(&self, id: usize, level: usize, ctx: &C, mmu: &SoftMmu);
}
