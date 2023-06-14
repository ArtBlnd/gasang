use core::ir::BasicBlock;

use super::Analysis;

pub struct IrCostAnalysis {
    pub basic_block: BasicBlock,
}

impl Analysis for IrCostAnalysis {
    type Output = IrCost;

    fn analyze(&self) -> Self::Output {
        todo!()
    }
}

pub struct IrCost {
    pub cost: u64,
}
