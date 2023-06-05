pub mod cranelift;
mod executable;

pub mod analysis;
pub mod interrupt;
pub mod rustjit;

use core::{ir::BasicBlock, Architecture};
pub use executable::*;

pub trait Codegen {
    type Context;
    type Executable: Executable<Context = Self::Context>;

    fn new_context<A: Architecture>() -> Self::Context;
    fn compile(&self, bb: BasicBlock) -> Self::Executable;
}
