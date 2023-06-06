use crate::ir::BasicBlock;

/// The representation of a machine instruction
pub trait Instruction: Sized {
    fn size(&self) -> u64;

    /// decode the instruction from raw bytes
    fn decode(raw_inst: &[u8]) -> Option<Self>;

    /// compile the instruction to IR
    fn compile_to_ir(&self, basic_block: &mut BasicBlock);
}
