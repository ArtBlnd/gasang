
use std::sync::Arc;
use std::cell::UnsafeCell;

#[derive(Clone)]
pub struct HostMemory {
    memory: Arc<UnsafeCell<Box<[u8]>>>,
}

impl HostMemory {
    pub fn new(size: usize) -> Self {
        assert!(size % 4096 == 0, "size must be a multiple of 4096");

        let mut memory = Vec::with_capacity(size);
        memory.resize(size, 0);
        HostMemory {
            memory: Arc::new(UnsafeCell::new(memory.into_boxed_slice())),
        }
    }

    pub unsafe fn get_slice(&self) -> &mut [u8] {
        unsafe { &mut *self.memory.get() }
    }
}