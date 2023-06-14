use core::ir::{BasicBlock, IrInst, IrValue};
use std::collections::HashSet;

use super::Analysis;

pub struct VariableLivenessAnalysis<'bb> {
    basic_block: &'bb BasicBlock,
}

impl<'bb> VariableLivenessAnalysis<'bb> {
    pub fn new(basic_block: &'bb BasicBlock) -> Self {
        Self { basic_block }
    }
}

pub struct VariableLiveness {
    killed: Vec<HashSet<IrValue>>,
    maximum_variable_live: usize,
}

impl VariableLiveness {
    /// Check if a variable is dead after an instruction index.
    pub fn is_dead_after(&self, inst_idx: usize, value: &IrValue) -> bool {
        assert!(matches!(value, IrValue::Variable(..)));
        self.killed[..inst_idx]
            .iter()
            .rev()
            .any(|killed| killed.contains(value))
    }

    /// Get the maximum number of variables live at any point in the basic block.
    pub fn maximum_variable_live(&self) -> usize {
        self.maximum_variable_live
    }
}

impl Analysis for VariableLivenessAnalysis<'_> {
    type Output = VariableLiveness;

    fn analyze(&self) -> Self::Output {
        let mut killed: Vec<HashSet<IrValue>> = Vec::new();
        let mut is_variable_used = HashSet::new();
        killed.resize_with(self.basic_block.inst().len(), Default::default);

        let mut try_mark_as_dead = |idx: usize, value: IrValue| {
            if let IrValue::Variable(..) = value {
                if is_variable_used.contains(&value) {
                    is_variable_used.insert(value);
                    killed[idx].insert(value);
                }
            }
        };

        for (idx, inst) in self.basic_block.inst().iter().enumerate().rev() {
            match inst {
                &IrInst::Add { dst, lhs, rhs }
                | &IrInst::Sub { dst, lhs, rhs }
                | &IrInst::Mul { dst, lhs, rhs }
                | &IrInst::Div { dst, lhs, rhs }
                | &IrInst::Rem { dst, lhs, rhs }
                | &IrInst::BitAnd { dst, lhs, rhs }
                | &IrInst::BitOr { dst, lhs, rhs }
                | &IrInst::BitXor { dst, lhs, rhs }
                | &IrInst::LogicalAnd { dst, lhs, rhs }
                | &IrInst::LogicalXor { dst, lhs, rhs }
                | &IrInst::LogicalOr { dst, lhs, rhs }
                | &IrInst::Shl { dst, lhs, rhs }
                | &IrInst::Lshr { dst, lhs, rhs }
                | &IrInst::Ashr { dst, lhs, rhs } => {
                    try_mark_as_dead(idx, dst);
                    try_mark_as_dead(idx, lhs);
                    try_mark_as_dead(idx, rhs);
                }
                &IrInst::LogicalNot { dst, src }
                | &IrInst::BitNot { dst, src }
                | &IrInst::Assign { dst, src }
                | &IrInst::Load { dst, src, .. }
                | &IrInst::Store { dst, src, .. }
                | &IrInst::ZextCast { dst, src }
                | &IrInst::SextCast { dst, src } => {
                    try_mark_as_dead(idx, dst);
                    try_mark_as_dead(idx, src);
                }
                IrInst::Fence { .. } | IrInst::MoveFlag { .. } | IrInst::Interrupt(_) => {}
                IrInst::Intrinsic(_) => todo!(),
            }
        }

        let mut maximum_variable_live = 0;
        let mut variable_live: HashSet<IrValue> = HashSet::new();

        let try_mark_as_live = |value: IrValue, variable_live: &mut HashSet<IrValue>| {
            if let IrValue::Variable(..) = value {
                if !variable_live.contains(&value) {
                    variable_live.insert(value);
                }
            }
        };

        for (idx, inst) in self.basic_block.inst().iter().enumerate() {
            match inst {
                &IrInst::Add { dst, lhs, rhs }
                | &IrInst::Sub { dst, lhs, rhs }
                | &IrInst::Mul { dst, lhs, rhs }
                | &IrInst::Div { dst, lhs, rhs }
                | &IrInst::Rem { dst, lhs, rhs }
                | &IrInst::BitAnd { dst, lhs, rhs }
                | &IrInst::BitOr { dst, lhs, rhs }
                | &IrInst::BitXor { dst, lhs, rhs }
                | &IrInst::LogicalAnd { dst, lhs, rhs }
                | &IrInst::LogicalXor { dst, lhs, rhs }
                | &IrInst::LogicalOr { dst, lhs, rhs }
                | &IrInst::Shl { dst, lhs, rhs }
                | &IrInst::Lshr { dst, lhs, rhs }
                | &IrInst::Ashr { dst, lhs, rhs } => {
                    try_mark_as_live(dst, &mut variable_live);
                    try_mark_as_live(lhs, &mut variable_live);
                    try_mark_as_live(rhs, &mut variable_live);

                    // Remove dead variables
                    for value in &killed[idx] {
                        variable_live.remove(value);
                    }

                    maximum_variable_live = maximum_variable_live.max(variable_live.len());
                }
                &IrInst::LogicalNot { dst, src }
                | &IrInst::BitNot { dst, src }
                | &IrInst::Assign { dst, src }
                | &IrInst::Load { dst, src, .. }
                | &IrInst::Store { dst, src, .. }
                | &IrInst::ZextCast { dst, src }
                | &IrInst::SextCast { dst, src } => {
                    try_mark_as_live(dst, &mut variable_live);
                    try_mark_as_live(src, &mut variable_live);

                    // Remove dead variables
                    for value in &killed[idx] {
                        variable_live.remove(value);
                    }

                    maximum_variable_live = maximum_variable_live.max(variable_live.len());
                }
                IrInst::Fence { .. } | IrInst::MoveFlag { .. } | IrInst::Interrupt(_) => {}
                IrInst::Intrinsic(_) => todo!(),
            }
        }

        VariableLiveness {
            killed,
            maximum_variable_live,
        }
    }
}
