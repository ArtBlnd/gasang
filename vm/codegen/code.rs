use crate::codegen::Value;
use crate::Cpu;

pub trait CompiledCode: for<'a> Fn(&'a Cpu) -> Value {
    unsafe fn execute(&self, vm: &Cpu) -> Value;
}

impl<T> CompiledCode for T
where
    for<'a> T: Fn(&'a Cpu) -> Value,
{
    unsafe fn execute<'vm>(&self, vm: &'vm Cpu) -> Value {
        self(vm)
    }
}
