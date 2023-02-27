mod cache;
pub use cache::*;
mod executable;
pub use executable::*;

pub mod cranelift;
pub mod flag_policy;
pub mod rustjit;

use crate::ir::{Ir, IrBlock};
use crate::value::Value;

pub trait Codegen {
    type Exec: Executable<Output = Value>;
    type ExecBlock: Executable;

    // compile ir into executable object.
    fn compile_ir(&self, ir: &Ir) -> Self::Exec;

    // compile ir block into executable object.
    fn compile_ir_block(&self, ir: &IrBlock) -> Self::ExecBlock;
}
