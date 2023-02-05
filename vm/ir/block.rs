use smallvec::SmallVec;

use crate::ir::{Ir, Type};

use crate::register::RegId;

#[derive(Clone, Debug)]
pub enum BlockDestination {
    Flags,
    Eip,
    GprRegister(RegId),
    FprRegister(RegId),
    Memory(u64),
    None,
    SystemCall,
    Exit,
}

#[derive(Clone, Debug)]
pub struct IrBlock {
    items: SmallVec<[IrBlockItem; 2]>,

    original_size: usize,
}

impl IrBlock {
    pub fn new(original_size: usize) -> Self {
        Self {
            items: SmallVec::new(),
            original_size,
        }
    }

    pub fn append(&mut self, ir: Ir, dest: BlockDestination) {
        self.items.push(IrBlockItem {
            ir_root: ir,
            ir_dest: dest,
        });
    }

    pub fn items(&self) -> &[IrBlockItem] {
        &self.items
    }

    pub fn original_size(&self) -> usize {
        self.original_size
    }
}

#[derive(Clone, Debug)]
pub struct IrBlockItem {
    ir_root: Ir,
    ir_dest: BlockDestination,
}

impl IrBlockItem {
    pub fn root(&self) -> &Ir {
        &self.ir_root
    }

    pub fn dest(&self) -> &BlockDestination {
        &self.ir_dest
    }
}
