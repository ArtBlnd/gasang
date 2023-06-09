use crate::codegen::Context;

mod aarch64_unknown_linux;
pub use aarch64_unknown_linux::*;
use device::IoDevice;

pub trait Abi {
    fn new() -> Self;

    /// Initialize the ABI with the given binary.
    fn on_initialize<C: Context, M: IoDevice>(&mut self, binary: &[u8], ctx: &C, mmu: &M);
    /// Called when an exception occurs.
    fn on_exception<C: Context, M: IoDevice>(&self, exception: u64, ctx: &C, mmu: &M);
    /// Called when an interrupt occurs.
    fn on_interrupt<C: Context, M: IoDevice>(&self, interrupt: u64, ctx: &C, mmu: &M);
    /// Called when a system call occurs.
    fn on_system_call<C: Context, M: IoDevice>(&self, system_call: u64, ctx: &C, mmu: &M);
    /// Called when an IRQ occurs.
    fn on_irq<C: Context, M: IoDevice>(&self, id: usize, level: usize, ctx: &C, mmu: &M);
}
