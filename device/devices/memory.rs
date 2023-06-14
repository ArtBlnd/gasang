use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::IoDevice;

/// A simple memory device.
#[derive(Clone)]
pub struct Memory {
    mem: Arc<UnsafeCell<Box<[u8]>>>,
}

impl Memory {
    pub fn allocate(size: usize) -> Self {
        Self {
            mem: Arc::new(UnsafeCell::new(vec![0; size].into_boxed_slice())),
        }
    }

    unsafe fn view(&self) -> &mut [u8] {
        &mut *self.mem.get()
    }
}

impl IoDevice for Memory {
    unsafe fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        let mem = self.view();
        let len = buf.len().min(mem.len() - offset as usize);
        buf[..len].copy_from_slice(&mem[offset as usize..offset as usize + len]);
        len
    }

    unsafe fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        let mem = self.view();
        let len = buf.len().min(mem.len() - offset as usize);
        mem[offset as usize..offset as usize + len].copy_from_slice(&buf[..len]);
        len
    }
}
