pub mod aarch64;

use crate::ir::Block;
use crate::error::CompileError;

pub trait Compiler {
    type Item;

    fn compile(&self, item: Self::Item) -> Result<Block, CompileError>;
}