mod cache;
pub use cache::*;
mod executable;
pub use executable::*;
mod prelude;
pub use prelude::*;

pub mod cranelift;
pub mod interpret;

use crate::error::CodegenError;

pub trait Codegen {
    type Executable: Executable;

    fn compile(&self, blocks: Vec<crate::ir::IrBlock>) -> Result<Self::Executable, CodegenError>;
}
