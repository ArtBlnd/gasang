use thiserror::Error;

#[derive(Debug, Error)]
pub enum Interrupt {
    #[error("Integer overflow")]
    IntegerOverflow,

    #[error("Interrupting with value {0}")]
    Interrupt(usize)
}
