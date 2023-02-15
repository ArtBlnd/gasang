mod aarch64_flag_policy;
pub use aarch64_flag_policy::*;
mod dummy;
pub use dummy::*;

use crate::ir::Type;
use crate::Cpu;

pub trait FlagPolicy {
    fn carry(&self, vm: &Cpu) -> bool;

    fn add_carry(&self, ty: Type, a: u64, b: u64, vm: &Cpu);
    fn sub_carry(&self, ty: Type, a: u64, b: u64, vm: &Cpu);
}
