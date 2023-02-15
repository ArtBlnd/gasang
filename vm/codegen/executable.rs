use crate::interrupt::InterruptModel;
use crate::Cpu;

pub trait Executable {
    unsafe fn execute(&self, vm: &mut Cpu, interrupt_mode: &dyn InterruptModel);
}
