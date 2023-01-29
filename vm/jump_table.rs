use slab::Slab;

use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, Copy)]
pub struct JumpId(u32);

impl Display for JumpId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "jumpid:{}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Checkpoint {
    pub ipr: u64,
    pub ipv: usize,
}

#[derive(Debug)]
pub struct JumpTable {
    checkpoint_min_distance: u64,

    ipv_table: Slab<usize>,
    ipr_checkpoints: HashMap<u64, Checkpoint>,
}

impl JumpTable {
    pub fn new(checkpoint_min_distance: u64) -> Self {
        Self {
            checkpoint_min_distance,

            ipv_table: Slab::new(),
            ipr_checkpoints: HashMap::new(),
        }
    }

    pub fn new_jump(&mut self, ipv: usize) -> JumpId {
        JumpId(self.ipv_table.insert(ipv) as u32)
    }

    pub fn jumpid2ipv(&self, jumpid: JumpId) -> usize {
        self.ipv_table[jumpid.0 as usize]
    }

    pub fn checkpoint_min_distance(&self) -> u64 {
        self.checkpoint_min_distance
    }

    pub fn new_checkpoint(&mut self, ipr: u64, ipv: usize) {
        let cp_offset = ipr / self.checkpoint_min_distance;
        self.ipr_checkpoints
            .insert(cp_offset, Checkpoint { ipr, ipv });
    }

    pub fn get_checkpoint(&self, ipr: u64) -> Checkpoint {
        let mut cp_offset = ipr / self.checkpoint_min_distance;

        loop {
            let Some(cp) = self.ipr_checkpoints.get(&cp_offset) else {
                panic!("Bad instruction size and its checkpoint!");
            };

            if cp.ipr > ipr {
                return *cp;
            }

            cp_offset += 1;
        }
    }
}
