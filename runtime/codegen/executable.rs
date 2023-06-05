use std::ops::Generator;

use device::devices::SoftMmu;

use super::interrupt::Interrupt;

/// An executable object that can be executed on a context.
pub trait Executable {
    type Context;
    type Generator<'a>: Generator<Yield = Interrupt> + 'a
    where
        Self: 'a;

    unsafe fn execute<'a>(
        &'a self,
        context: &'a mut Self::Context,
        io_device: &'a SoftMmu,
    ) -> Self::Generator<'a>;
}
