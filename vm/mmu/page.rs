use crate::mmu::HostMemory;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Page {
    Unmapped,
    Memory {
        memory: HostMemory,

        readable: bool,
        writable: bool,
        executable: bool,
    },
}
