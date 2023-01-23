use crate::mmu::HostMemory;

use std::sync::Arc;

#[derive(Clone)]
pub enum Page {
    Unmapped,
    Memory {
        memory: HostMemory,

        readable: bool,
        writable: bool,
        executable: bool,
    },
    Callback {
        callback: Arc<dyn FnMut(&mut [u8])>,
    },
}
