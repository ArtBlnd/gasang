use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crossbeam_queue::ArrayQueue;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Irq {
    pub id: usize,
    pub level: usize,
}

impl PartialOrd for Irq {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Irq {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level.cmp(&other.level)
    }
}

#[derive(Clone)]
pub struct IrqQueue {
    id_idx: Arc<AtomicUsize>,
    queue: Arc<ArrayQueue<Irq>>,
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
            level: AtomicUsize::new(0),
            queue: self.queue.clone(),
        };

        irq
    }

    pub fn recv(&self) -> Option<Irq> {
        self.queue.pop()
    }
}

pub struct DeviceIrqQueue {
    id: usize,
    level: AtomicUsize,
    queue: Arc<ArrayQueue<Irq>>,
}

impl DeviceIrqQueue {
    /// Send an interrupt to the device
    pub fn send(&self) {
        self.queue
            .push(Irq {
                id: self.id,
                level: self.level.load(Ordering::SeqCst),
            })
            .unwrap();
    }

    pub fn raise(&self) {
        self.level.fetch_add(1, Ordering::SeqCst);
        self.send();
    }

    pub fn lower(&self) {
        self.level.fetch_sub(1, Ordering::SeqCst);
    }
}
