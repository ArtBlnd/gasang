use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crossbeam_queue::ArrayQueue;

#[derive(Clone)]
pub struct IrqQueue {
    id_idx: Arc<AtomicUsize>,
    queue: Arc<ArrayQueue<usize>>,
}

impl IrqQueue {
    /// Create a new IRQ object
    pub fn new() -> Self {
        Self {
            id_idx: Arc::new(AtomicUsize::new(0)),
            queue: Arc::new(ArrayQueue::new(1024)),
        }
    }

    /// Create a new irq object for the device
    pub fn issue_sender(&self) -> DeviceIrqQueue {
        let irq = DeviceIrqQueue {
            id: self.id_idx.fetch_add(1, Ordering::SeqCst),
            queue: self.queue.clone(),
        };

        irq
    }

    pub fn recv(&self) -> Option<usize> {
        self.queue.pop()
    }
}

pub struct DeviceIrqQueue {
    id: usize,
    queue: Arc<ArrayQueue<usize>>,
}

impl DeviceIrqQueue {
    /// Send an interrupt to the device
    pub fn send(&self) {
        self.queue.push(self.id).unwrap();
    }
}
