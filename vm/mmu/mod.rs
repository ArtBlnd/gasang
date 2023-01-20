mod frame;
pub use frame::*;
mod page;
pub use page::*;

use crate::error::MMUError;

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct MemoryManagementUnit {
    inner: Arc<MemoryManagementUnitInner>
}

impl Deref for MemoryManagementUnit {
    type Target = MemoryManagementUnitInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MemoryManagementUnit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner).unwrap()
    }
}

impl MemoryManagementUnit {
    pub unsafe fn frame(&self, addr: usize) -> Result<Frame, MMUError> {
        Ok(Frame::new(self.inner.clone(), addr))
    }
}

pub struct MemoryManagementUnitInner {
    // linear page table.
    page_size: usize,
    page_table: Box<[Page]>
}

impl MemoryManagementUnitInner {
    pub fn new(size: usize, page_size: usize, page_cnts: usize) -> Self {
        MemoryManagementUnitInner {
            page_size,
            page_table: vec![Page::Unmapped; page_cnts].into_boxed_slice(),
        }
    }

    pub fn query_mut(&mut self, addr: usize) -> Result<&mut Page, MMUError> {
        let page_id = addr / self.page_size;
        let Some(page) = self.page_table.get_mut(page_id) else {
            return Err(MMUError::PageNotMapped);
        };

        Ok(page)
    }

    pub fn query(&self, addr: usize) -> Result<&Page, MMUError> {
        let page_id = addr / self.page_size;
        let Some(page) = self.page_table.get(page_id) else {
            return Err(MMUError::PageNotMapped);
        };

        Ok(page)
    }
}