use crate::mmu::MemoryManagementUnitInner;

use std::sync::Arc;
use std::io::{Read, Write};

pub enum Frame {
    Memory {
        mmu_access: Arc<MemoryManagementUnitInner>,
        offs: usize,
    },
}