mod aarch64_flag_policy;

pub use aarch64_flag_policy::*;
mod dummy;
pub use dummy::*;

use crate::ir::Type;
use crate::Cpu;

use std::sync::Arc;

pub trait FlagPolicy {
    fn carry(&self, vm: &Cpu) -> bool;

    fn add_carry(&self, ty: Type, a: u64, b: u64, vm: &Cpu);
    fn sub_carry(&self, ty: Type, a: u64, b: u64, vm: &Cpu);
}

impl<T> FlagPolicy for Arc<T>
where
    T: FlagPolicy + ?Sized,
{
    fn carry(&self, vm: &Cpu) -> bool {
        self.as_ref().carry(vm)
    }

    fn add_carry(&self, ty: Type, a: u64, b: u64, vm: &Cpu) {
        self.as_ref().add_carry(ty, a, b, vm)
    }

    fn sub_carry(&self, ty: Type, a: u64, b: u64, vm: &Cpu) {
        self.as_ref().sub_carry(ty, a, b, vm)
    }
}

impl<T> FlagPolicy for Box<T>
where
    T: FlagPolicy + ?Sized,
{
    fn carry(&self, vm: &Cpu) -> bool {
        self.as_ref().carry(vm)
    }

    fn add_carry(&self, ty: Type, a: u64, b: u64, vm: &Cpu) {
        self.as_ref().add_carry(ty, a, b, vm)
    }

    fn sub_carry(&self, ty: Type, a: u64, b: u64, vm: &Cpu) {
        self.as_ref().sub_carry(ty, a, b, vm)
    }
}
