use crate::vm_state::VmState;

pub trait Executable {
    unsafe fn execute(&self, vm: &mut VmState);
}
