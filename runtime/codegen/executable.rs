use core::Interrupt;
use std::ops::Generator;

use crate::SoftMmu;

use super::Context;

/// An executable object that can be executed on a context.
pub trait Executable {
    type Context: Context;
    type Generator<'a>: Generator<Yield = Interrupt> + 'a
    where
        Self: 'a;

    unsafe fn execute<'a>(
        &'a self,
        context: &'a mut Self::Context,
        mmu: &'a SoftMmu,
    ) -> Self::Generator<'a>;
}
