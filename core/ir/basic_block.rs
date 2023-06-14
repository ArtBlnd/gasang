use std::collections::HashMap;

use super::{IrInst, IrType, IrValue};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BasicBlock {
    pub(crate) addr: u64,

    pub(crate) statements: Vec<IrInst>,
    pub(crate) terminator: BasicBlockTerminator,
    pub(crate) variable_count: usize,
    pub(crate) varaibles: HashMap<usize, IrType>,
}

impl BasicBlock {
    /// Create a new basic block with the given address
    pub fn new(addr: u64) -> Self {
        Self {
            addr,
            ..Default::default()
        }
    }

    pub fn inst(&self) -> &[IrInst] {
        &self.statements
    }

    pub fn push_inst(&mut self, statement: IrInst) {
        self.statements.push(statement);
    }

    pub fn terminator(&self) -> BasicBlockTerminator {
        self.terminator
    }

    pub fn set_terminator(&mut self, terminator: BasicBlockTerminator) {
        self.terminator = terminator;
    }

    pub fn new_variable(&mut self, ty: IrType) -> IrValue {
        let variable = IrValue::Variable(ty, self.variable_count);
        self.variable_count += 1;
        variable
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum BasicBlockTerminator {
    #[default]
    None,
    /// Branch to the next basic block
    Next,
    /// Branch to the basic block if the condition is true
    BranchCond { cond: IrValue, target: IrValue },
    /// Branch to another basic block if the condition is true
    Branch(IrValue),
}
