#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Reordering {
    /// Relaxed ordering, no guarantees.
    Relaxed,
    /// Acquire ordering, all previous writes are visible.
    Acquire,
    /// Release ordering, all subsequent writes are visible.
    Release,
    /// Acquire and release ordering, all previous and subsequent writes are visible.
    SeqCst,
}
