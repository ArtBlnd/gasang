mod host_memory;
pub use host_memory::*;
mod memory_frame;
pub use memory_frame::*;
mod page;
pub use page::*;

use crate::error::MMUError;

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

const PAGE_SIZE: u64 = 65536;

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
    pub fn frame(&self, addr: u64) -> MemoryFrame {
        MemoryFrame::new(self.inner.clone(), addr)
    }
}

// Internal Memory Management Unit Data
//
// This is the internal data of the Mmu. It is used to implement the Deref and
// DerefMut traits. It is not recommended to use this struct directly.
#[derive(Debug)]
pub struct MmuData {
    page_table: RwLock<PageTable>,
}

impl MmuData {
    pub fn new() -> Self {
        MmuData {
            page_table: RwLock::new(PageTable::new()),
        }
    }

    pub fn query(&self, addr: u64) -> Result<Page, MMUError> {
        let pt = self.page_table.read().unwrap();

        let Some(page) = pt.get_ref(addr) else {
            return Err(MMUError::PageFault);
        };

        Ok(page.clone())
    }

    pub fn mmap(&self, addr: u64, size: u64, readable: bool, writable: bool, executable: bool) {
        let mut pt = self.page_table.write().unwrap();

        let mut offset = 0u64;

        while size > offset {
            let page = pt.get_or_mmap(addr + offset, || Page::Memory {
                memory: HostMemory::new(PAGE_SIZE as usize),
                readable,
                writable,
                executable,
            });

            let r = readable;
            let w = writable;
            let e = executable;

            if let Page::Memory {
                memory,
                readable,
                writable,
                executable,
            } = page
            {
                *readable = r;
                *writable = w;
                *executable = e;
            }

            offset += PAGE_SIZE;
        }
    }
}
