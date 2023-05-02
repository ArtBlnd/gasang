use std::cell::UnsafeCell;
// Host Memory
//
// This is the memory that is allocated by the host. It is used to store the
// guest memory as unsafe way. Note that accessing the memory outside of VM
// is very dangerous.
#[derive(Debug)]
pub struct HostMemory {
    memory: UnsafeCell<Box<[u8]>>,
}

impl HostMemory {
    pub fn new(size: usize) -> Self {
        assert!(size % 4096 == 0, "size must be a multiple of 4096");

        let mut memory = Vec::with_capacity(size);
        memory.resize(size, 0);
        HostMemory {
            memory: UnsafeCell::new(memory.into_boxed_slice()),
        }
    }

    pub unsafe fn slice(&self) -> &mut [u8] {
        unsafe { &mut *self.memory.get() }
    }
}

impl Clone for HostMemory {
    fn clone(&self) -> Self {
        let mut memory = Vec::new();
        unsafe {
            memory.extend_from_slice(self.slice());
        }

        let memory = UnsafeCell::new(memory.into_boxed_slice());
        Self { memory }
    }
}
