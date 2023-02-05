pub mod aarch64;

use crate::error::CompileError;
use crate::ir::Block;

pub trait Compiler {
    type Item;

    fn compile(&self, item: Self::Item) -> Result<Block, CompileError>;
}
