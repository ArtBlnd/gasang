use crate::register::RegId;
use crate::mmu::MemoryFrame;
use crate::ir::Ir;

pub enum BlockDestination {
    Flags,
    Eip,
    Register(RegId),
    Memory(MemoryFrame),
}

pub struct Block {
    ir_root: Ir,
    ir_dest: BlockDestination,

    original_size: usize
}

impl Block {
    pub fn new(ir_root: Ir, ir_dest: BlockDestination, original_size: usize) -> Self {
        Self {
            ir_root,
            ir_dest,
            original_size
        }
    }

    pub fn ir_root(&self) -> &Ir {
        &self.ir_root
    }

    pub fn ir_dest(&self) -> &BlockDestination {
        &self.ir_dest
    }

    pub fn original_size(&self) -> usize {
        self.original_size
    }
}