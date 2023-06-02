/// An executable object that can be executed on a context.
pub trait Executable {
    type Context;

    unsafe fn execute(&self, context: &mut Self::Context);
}
