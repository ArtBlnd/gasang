use crate::mmu::{MmuData, Page, PAGE_SIZE};
use crate::MMUError;

use std::sync::Arc;

pub struct MemoryFrame {
    mmu: Arc<MmuData>,
    addr: u64,
}

impl MemoryFrame {
    pub fn new(mmu: Arc<MmuData>, addr: u64) -> Self {
        MemoryFrame {
            mmu: mmu.clone(),
            addr,
        }
    }

    pub unsafe fn read(&mut self, buf: &mut [u8]) -> Result<(), MMUError> {
        let mut addr = self.addr;
        let mut read = 0;

        while read < buf.len() {
            let page = self.mmu.query(addr)?;
            let page_offs_beg = (addr % PAGE_SIZE) as usize;
            let page_offs_end = (usize::min(PAGE_SIZE as usize, buf.len() - read)) as usize;

            match page {
                Page::Unmapped => return Err(MMUError::PageNotMapped),
                Page::Memory {
                    memory, readable, ..
                } => {
                    // Check if the page is readable
                    if !readable {
                        return Err(MMUError::AccessViolation);
                    }

                    let mem = unsafe {
                        &mut memory.get_slice()[page_offs_beg..page_offs_beg + page_offs_end]
                    };
                    buf.copy_from_slice(mem);
                    read += mem.len();
                    addr += mem.len() as u64;
                }
            }
        }

        Ok(())
    }

    pub unsafe fn write(&mut self, buf: &[u8]) -> Result<(), MMUError> {
        let mut addr = self.addr;
        let mut writ = 0;

        while writ < buf.len() {
            let page = self.mmu.query(addr)?;
            let page_offs_beg = (addr % PAGE_SIZE) as usize;
            let page_offs_end = (usize::min(PAGE_SIZE as usize, buf.len() - writ)) as usize;

            match page {
                Page::Unmapped => return Err(MMUError::PageNotMapped),
                Page::Memory {
                    memory, writable, ..
                } => {
                    // Check if the page is writable
                    if !writable {
                        return Err(MMUError::AccessViolation);
                    }

                    let mem = unsafe {
                        &mut memory.get_slice()[page_offs_beg..page_offs_beg + page_offs_end]
                    };
                    mem.copy_from_slice(buf);
                    writ += mem.len();
                    addr += mem.len() as u64;
                }
            }
        }

        Ok(())
    }

    pub unsafe fn read_u8(&mut self) -> Result<u8, MMUError> {
        let mut buf = [0u8; 1];
        self.read(&mut buf)?;
        Ok(buf[0])
    }

    pub unsafe fn read_u16(&mut self) -> Result<u16, MMUError> {
        let mut buf = [0u8; 2];
        self.read(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub unsafe fn read_u32(&mut self) -> Result<u32, MMUError> {
        let mut buf = [0u8; 4];
        self.read(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub unsafe fn read_u64(&mut self) -> Result<u64, MMUError> {
        let mut buf = [0u8; 8];
        self.read(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    pub unsafe fn write_u8(&mut self, val: u8) -> Result<(), MMUError> {
        self.write(&[val])
    }

    pub unsafe fn write_u16(&mut self, val: u16) -> Result<(), MMUError> {
        self.write(&val.to_le_bytes())
    }

    pub unsafe fn write_u32(&mut self, val: u32) -> Result<(), MMUError> {
        self.write(&val.to_le_bytes())
    }

    pub unsafe fn write_u64(&mut self, val: u64) -> Result<(), MMUError> {
        self.write(&val.to_le_bytes())
    }
}
