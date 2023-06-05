use std::cell::RefCell;
use thread_local::ThreadLocal;

use crate::io_device::IoDevice;

pub struct SoftMmu {
    map: Vec<Mapping>,
    last_access: ThreadLocal<RefCell<LastAccess>>,
}

impl SoftMmu {
    pub fn get_mapping_index(&self, addr: u64) -> Option<usize> {
        // We use a thread local to cache the last access to speed up the common case of
        // sequential accesses.
        self.get_index_fast(addr)
            .or_else(|| self.get_index_slow(addr))
    }

    pub fn get_index_slow(&self, addr: u64) -> Option<usize> {
        for (index, mapping) in self.map.iter().enumerate() {
            if mapping.addr <= addr && addr < mapping.addr + mapping.size {
                let new_last_access = LastAccess {
                    addr: mapping.addr,
                    size: mapping.size,
                    index,
                };

                let last_access = self.last_access.get_or(|| RefCell::new(new_last_access));
                *last_access.borrow_mut() = new_last_access;

                return Some(index);
            }
        }
        None
    }

    pub fn get_index_fast(&self, addr: u64) -> Option<usize> {
        let last_access = self.last_access.get()?.borrow();
        if last_access.addr <= addr && addr < last_access.addr + last_access.size {
            return Some(last_access.index);
        }
        None
    }

    /// map a new IO device into the address space
    pub fn map(&mut self, io: impl IoDevice + 'static, addr: u64, size: u64) {
        self.map.push(Mapping {
            io: Box::new(io),
            addr,
            size,
        });
    }
}

impl IoDevice for SoftMmu {
    unsafe fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        let idx = self
            .get_mapping_index(offset)
            .expect("Tried to read from unmapped memory");

        let mapping = &self.map[idx];
        mapping.io.read_at(offset - mapping.addr, buf)
    }

    unsafe fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        let idx = self
            .get_mapping_index(offset)
            .expect("Tried to read from unmapped memory");

        let mapping = &self.map[idx];
        mapping.io.write_at(offset - mapping.addr, buf)
    }
}

#[derive(Clone, Copy)]
pub struct LastAccess {
    addr: u64,
    size: u64,
    index: usize,
}

pub struct Mapping {
    io: Box<dyn IoDevice>,
    addr: u64,
    size: u64,
}
