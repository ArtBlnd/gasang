mod cache;
pub use cache::*;
mod executable;
pub use executable::*;
mod prelude;
pub use prelude::*;

mod cranelift;
mod interpret;

use crate::error::CodegenError;

pub trait Codegen {
    type Executable: Executable;

    fn compile(&self, blocks: Vec<crate::ir::Block>) -> Result<Self::Executable, CodegenError>;
}
