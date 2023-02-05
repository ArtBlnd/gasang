use crate::VmState;

pub trait InterpretFunc: for<'a> Fn(&'a mut VmState) -> u64 {
    unsafe fn execute(&self, vm: &mut VmState) -> u64;
}

impl<T> InterpretFunc for T
where
    for<'a> T: Fn(&'a mut VmState) -> u64,
{
    unsafe fn execute<'vm>(&self, vm: &'vm mut VmState) -> u64 {
        self(vm)
    }
}
