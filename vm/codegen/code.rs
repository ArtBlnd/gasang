use crate::VmState;

pub trait CompiledCode: for<'a> Fn(&'a VmState) -> u64 {
    unsafe fn execute(&self, vm: &VmState) -> u64;
}

impl<T> CompiledCode for T
where
    for<'a> T: Fn(&'a VmState) -> u64,
{
    unsafe fn execute<'vm>(&self, vm: &'vm VmState) -> u64 {
        self(vm)
    }
}
