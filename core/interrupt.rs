/// Interrupts that can be raised by the runtime.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Interrupt {
    SystemCall(u64),
    Aborts(i32),
    Reset,
    Exception(u64),
    Interrupt(u64),
    Yield,
    WaitForInterrupt,
    DivideByZero,
}

pub struct InterruptQueue {
    queue: crossbeam_queue::ArrayQueue<Interrupt>,
}

impl InterruptQueue {
    pub fn new() -> Self {
        Self {
            queue: crossbeam_queue::ArrayQueue::new(1024),
        }
    }

    pub fn enqueue(&self, interrupt: Interrupt) {
        self.queue.push(interrupt).unwrap();
    }

    pub fn dequeue(&self) -> Option<Interrupt> {
        self.queue.pop()
    }
}
