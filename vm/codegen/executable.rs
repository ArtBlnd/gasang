use crate::interrupt::InterruptModel;
use crate::VmState;

pub trait Executable {
    unsafe fn execute(&self, vm: &mut VmState, interrupt_mode: &dyn InterruptModel);
}
