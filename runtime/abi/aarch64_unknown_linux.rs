use core::ArchitectureCompat;

use arch_desc::aarch64::AArch64Architecture;
use device::IoDevice;

use crate::codegen::Context;

use super::Abi;

pub struct AArch64UnknownLinux {}
impl ArchitectureCompat<AArch64Architecture> for AArch64UnknownLinux {}

impl Abi for AArch64UnknownLinux {
    fn new() -> Self {
        todo!()
    }

    fn on_initialize<C: Context, M: IoDevice>(&mut self, binary: &[u8], ctx: &C, mmu: &M) {
        todo!()
    }

    fn on_exception<C: Context, M: IoDevice>(&self, exception: u64, ctx: &C, mmu: &M) {
        todo!()
    }

    fn on_interrupt<C: Context, M: IoDevice>(&self, interrupt: u64, ctx: &C, mmu: &M) {
        todo!()
    }

    fn on_system_call<C: Context, M: IoDevice>(&self, system_call: u64, ctx: &C, mmu: &M) {
        todo!()
    }

    fn on_irq<C: Context, M: IoDevice>(&self, id: usize, level: usize, ctx: &C, mmu: &M) {
        todo!()
    }
}
