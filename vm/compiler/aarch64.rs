use crate::compiler::Compiler;
use crate::error::CompileError;
use crate::ir::*;
use crate::register::RegId;

use machineinstr::aarch64::AArch64Instr;

pub struct AArch64Compiler {
    gpr_registers: [RegId; 31],
    fpr_registers: [RegId; 31],
}

impl AArch64Compiler {
    pub fn gpr(&self, index: usize) -> RegId {
        self.gpr_registers[index]
    }

    pub fn fpr(&self, index: usize) -> RegId {
        self.fpr_registers[index]
    }
}

impl Compiler for AArch64Compiler {
    type Item = AArch64Instr;

    fn compile(&self, item: Self::Item) -> Result<Block, CompileError> {
        todo!()
    }
}
