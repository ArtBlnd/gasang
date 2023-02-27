use crate::softmmu::Mmu;
use crate::Cpu;

pub struct ExecutionContext<'a> {
    pub cpu: &'a mut Cpu,
    pub mmu: &'a Mmu,
}

pub trait Executable {
    type Output;

    unsafe fn execute<'a>(&self, ctx: &mut ExecutionContext<'a>) -> Self::Output;
}

pub struct FnExec<O> {
    func: Box<dyn for<'a> Fn(&mut ExecutionContext<'a>) -> O>,
}

impl<O> FnExec<O> {
    pub fn new<F>(func: F) -> Self
    where
        F: for<'a> Fn(&mut ExecutionContext<'a>) -> O + 'static,
    {
        Self {
            func: Box::new(func),
        }
    }
}

impl<O> Executable for FnExec<O> {
    type Output = O;

    unsafe fn execute<'a>(&self, ctx: &mut ExecutionContext<'a>) -> O {
        (self.func)(ctx)
    }
}
