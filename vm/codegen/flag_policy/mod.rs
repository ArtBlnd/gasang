mod aarch64_flag_policy;
pub use aarch64_flag_policy::*;
mod dummy;
pub use dummy::*;

use crate::ir::Type;
use crate::VmState;

pub trait FlagPolicy {
    fn carry(&self, vm: &VmState) -> bool;

    fn add_carry(&self, ty: Type, a: u64, b: u64, vm: &VmState);
    fn sub_carry(&self, ty: Type, a: u64, b: u64, vm: &VmState);
}
