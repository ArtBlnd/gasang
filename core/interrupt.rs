/// Interrupts that can be raised by the runtime.
pub enum Interrupt {
    Aborts(u64),
    Reset(u64),
    Exception(u64),
    Interrupt(u64),
}
