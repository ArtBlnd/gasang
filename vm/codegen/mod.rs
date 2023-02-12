mod cache;
pub use cache::*;
mod executable;
pub use executable::*;
mod prelude;
pub use prelude::*;
mod block_dest;
pub use block_dest::*;
mod block;
pub use block::*;
mod code;
pub use code::*;
mod value;
pub use value::*;

pub mod cranelift;
pub mod flag_policy;
pub mod interpret;

use crate::ir::Ir;

pub trait Codegen {
    type Code: CompiledCode;

    fn compile(&self, ir: Ir) -> Self::Code;
}
