use crate::Cpu;

pub trait Executable {
    unsafe fn execute(&self, vm: &mut Cpu);
}
