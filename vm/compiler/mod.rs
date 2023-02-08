pub mod aarch64;

use crate::error::CompileError;
use crate::ir::IrBlock;

pub trait Compiler {
    type Item;

    fn compile(&self, item: Self::Item) -> Result<IrBlock, CompileError>;
}
