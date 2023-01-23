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
    // Create a memory frame from a virtual address
    pub unsafe fn frame(&self, addr: usize) -> Result<MemoryFrame, MMUError> {
        Ok(MemoryFrame::new(self.inner.clone(), addr))
    }
}

// Internal Memory Management Unit Data
//
// This is the internal data of the Mmu. It is used to implement the Deref and 
// DerefMut traits. It is not recommended to use this struct directly.
pub struct MmuData {
    page_size: usize,
    page_table: Box<[Page]>,
}

impl MmuData {
    pub fn new(page_size: usize, page_cnts: usize) -> Self {
        MmuData {
            page_size,
            page_table: vec![Page::Unmapped; page_cnts].into_boxed_slice(),
        }
    }

    pub fn query(&self, addr: usize) -> Result<&Page, MMUError> {
        let page_id = addr / self.page_size;
        let Some(page) = self.page_table.get(page_id) else {
            return Err(MMUError::PageNotMapped);
        };

        Ok(page)
    }

    pub fn query_mut(&mut self, addr: usize) -> Result<&mut Page, MMUError> {
        let page_id = addr / self.page_size;
        let Some(page) = self.page_table.get_mut(page_id) else {
            return Err(MMUError::PageNotMapped);
        };

        Ok(page)
    }
}
