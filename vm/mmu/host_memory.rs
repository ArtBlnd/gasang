use std::cell::UnsafeCell;
use std::sync::Arc;


// Host Memory
//
// This is the memory that is allocated by the host. It is used to store the
// guest memory as unsafe way. Note that accessing the memory outside of VM 
// is very dangerous.
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

trait HostMemoryAllocator {
    fn alloc(&mut self, size: usize) -> HostMemory;
}
