pub mod cranelift;
mod executable;
pub use executable::*;
mod context;
pub use context::*;

pub mod analysis;
pub mod rustjit;

use core::{ir::BasicBlock, Architecture};

pub trait Codegen {
    type Context: Context;
    type Executable: Executable<Context = Self::Context>;

    fn new() -> Self;

    /// Allocate a new context for the given architecture.
    fn allocate_execution_context<A: Architecture>() -> Self::Context;
    fn compile<A: Architecture>(&self, bb: &BasicBlock) -> Self::Executable;
}
