use slab::Slab;

#[derive(Debug, Clone, Copy)]
pub struct JumpId(u8);

#[derive(Debug, Clone, Copy)]
pub struct Checkpoint {
    pub ipr: u64,
    pub ipv: usize,
}

#[derive(Debug)]
pub struct JumpTable {
    ipv_table: Slab<usize>,
    ipr_checkpoints: Box<[Checkpoint]>,
}

impl JumpTable {
    pub fn new() -> Self {
        Self {
            ipv_table: Slab::new(),
            ipr_checkpoints: Box::new([]),
        }
    }

    pub fn jumpid2ipv(&self, jumpid: JumpId) -> usize {
        self.ipv_table[jumpid.0 as usize]
    }

    pub fn get_checkpoint(&self, ipr: u64) -> Checkpoint {
        self.ipr_checkpoints[ipr as usize]
    }
}
