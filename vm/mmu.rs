use crate::error::MMUError;

use std::cell::UnsafeCell;
use std::sync::Arc;
use std::io::{Read, Write};


pub struct MemoryManagementUnit {
    memory: Arc<UnsafeCell<Box<[u8]>>>
}

impl MemoryManagementUnit {
    pub fn new(size: usize) -> Self {
        MemoryManagementUnit {
            memory: Arc::new(UnsafeCell::new(vec![0; size].into_boxed_slice()))
        }
    }

    pub unsafe fn get_frame(&self, offs: usize) -> Result<Frame, MMUError> {
        Ok(Frame {
            memory: self.memory.clone(),
            offs,
        })
    }
}

pub struct Frame {
    memory: Arc<UnsafeCell<Box<[u8]>>>,
    offs: usize,
}

impl Read for Frame {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let memory = unsafe { &mut *self.memory.get() };
        buf.copy_from_slice(&memory[self.offs..self.offs + buf.len()]);

        Ok(buf.len())
    }
}

impl Write for Frame {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let memory = unsafe { &mut *self.memory.get() };
        memory[self.offs..self.offs + buf.len()].copy_from_slice(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}