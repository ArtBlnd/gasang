use crate::interrupt::InterruptModel;
use crate::VmState;

pub struct NoModel;
impl InterruptModel for NoModel {
    unsafe fn syscall(&self, _int: u64, _vm: &VmState) {
        unimplemented!()
    }
}
