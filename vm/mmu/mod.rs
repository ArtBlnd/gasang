mod host_memory;
pub use host_memory::*;
mod memory_frame;
pub use memory_frame::*;
mod page;
pub use page::*;

use crate::error::MMUError;

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

// Memory Management Unit
//
// This is the core of the virtual memory system. It manages the mapping between
// virtual address and VM host address and callbacks. also manages the page table.
#[derive(Debug, Clone)]
pub struct Mmu {
    inner: Arc<MmuData>,
}

impl DerefMut for Mmu {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner).unwrap()
    }
}

impl Deref for Mmu {
    type Target = MmuData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Mmu {
    pub fn new() -> Self {
        Mmu {
            inner: Arc::new(MmuData::new()),
        }
    }

    // Create a memory frame from a virtual address
    pub fn frame(&self, addr: u64) -> Result<MemoryFrame, MMUError> {
        Ok(MemoryFrame::new(self.inner.clone(), addr))
    }
}

// Internal Memory Management Unit Data
//
// This is the internal data of the Mmu. It is used to implement the Deref and
// DerefMut traits. It is not recommended to use this struct directly.
#[derive(Debug)]
pub struct MmuData {
    page_table: PageTable,
}

impl MmuData {
    pub fn new() -> Self {
        MmuData {
            page_table: PageTable::new(),
        }
    }

    pub fn query(&self, addr: u64) -> Result<&Page, MMUError> {
        let Some(page) = self.page_table.get_ref(addr) else {
            return Err(MMUError::PageNotMapped);
        };

        Ok(page)
    }

    pub fn query_mut(&mut self, addr: u64) -> Result<&mut Page, MMUError> {
        let Some(page) = self.page_table.get_mut(addr) else {
            return Err(MMUError::PageNotMapped);
        };

        Ok(page)
    }
}
