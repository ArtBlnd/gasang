use core::ir::{IrConstant, IrValue};

pub trait Context {
    /// Speical function to get the value of a ir variable from outside of the vm world.
    fn evaluate(&self, value: IrValue) -> IrConstant;
}
