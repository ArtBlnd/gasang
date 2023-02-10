pub mod aarch64;
pub mod aarch64_prelude;

use crate::error::CompileError;
use crate::ir::IrBlock;

pub trait Compiler {
    type Item;

    fn compile(&self, item: Self::Item) -> IrBlock;
}
