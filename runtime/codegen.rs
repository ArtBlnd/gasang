pub mod cranelift;
mod executable;
pub use executable::*;
mod context;
pub use context::*;

pub mod analysis;
pub mod rustjit;

use core::{ir::BasicBlock, Architecture};

pub trait Codegen {
    type Context;
    type Executable: Executable<Context = Self::Context>;

    fn new_context<A: Architecture>() -> Self::Context;
    fn compile(&self, bb: &BasicBlock) -> Self::Executable;
}
