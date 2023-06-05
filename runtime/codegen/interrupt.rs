/// Interrupts that can be raised by the runtime.
pub enum Interrupt {
    Exception(u64),
}
