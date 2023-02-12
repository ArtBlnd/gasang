use crate::codegen::Value;
use crate::VmState;

pub trait CompiledCode: for<'a> Fn(&'a VmState) -> Value {
    unsafe fn execute(&self, vm: &VmState) -> Value;
}

impl<T> CompiledCode for T
where
    for<'a> T: Fn(&'a VmState) -> Value,
{
    unsafe fn execute<'vm>(&self, vm: &'vm VmState) -> Value {
        self(vm)
    }
}
