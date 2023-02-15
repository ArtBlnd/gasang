use smallvec::SmallVec;

use crate::ir::*;
use crate::register::RegId;

#[derive(Clone, Debug)]
pub enum BlockDestination {
    Flags,
    Ip,
    Gpr(Type, RegId),
    Fpr(Type, RegId),
    FprSlot(Type, RegId, u8),
    Memory(Type, u64),
    MemoryRelI64(Type, RegId, i64),
    MemoryRelU64(Type, RegId, u64),
    MemoryIr(Ir),
    None,
    SystemCall,
    Exit,
}

#[derive(Clone, Debug)]
pub struct IrBlock {
    items: SmallVec<[IrBlockItem; 2]>,
    original_size: usize,
    restore_flag: bool,
}

impl IrBlock {
    pub fn new(original_size: usize) -> Self {
        Self {
            items: SmallVec::new(),
            original_size,
            restore_flag: false,
        }
    }

    pub fn append(&mut self, ir: Ir, dest: BlockDestination) {
        let ir_type = ir.get_type();

        let dest_type = match &dest {
            BlockDestination::Flags => Some(&Type::U64),
            BlockDestination::Ip => Some(&Type::U64),
            BlockDestination::Gpr(ty, _) => Some(ty),
            BlockDestination::Fpr(ty, _) => Some(ty),
            BlockDestination::FprSlot(ty, _, _) => Some(ty),
            BlockDestination::Memory(ty, _) => Some(ty),
            BlockDestination::MemoryRelI64(ty, _, _) => Some(ty),
            BlockDestination::MemoryRelU64(ty, _, _) => Some(ty),
            BlockDestination::MemoryIr(_) => None,
            BlockDestination::None => None,
            BlockDestination::SystemCall => Some(&Type::U64),
            BlockDestination::Exit => None,
        };

        if let Some(dest_type) = dest_type {
            assert_eq!(&ir_type, dest_type);
        }

        self.items.push(IrBlockItem {
            ir_root: ir,
            ir_dest: dest,
        });
    }

    pub fn items(&self) -> &[IrBlockItem] {
        &self.items
    }

    pub fn restore_flag(&self) -> bool {
        self.restore_flag
    }

    pub fn set_restore_flag(&mut self) {
        self.restore_flag = true;
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
