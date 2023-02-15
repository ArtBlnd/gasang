mod aarch64_unix_model;
pub use aarch64_unix_model::*;
mod no_model;
pub use no_model::*;

use crate::Cpu;

pub trait InterruptModel {
    unsafe fn syscall(&self, int: u64, vm: &mut Cpu);
}
