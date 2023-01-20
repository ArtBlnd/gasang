use thiserror::Error;

#[derive(Clone, Copy, Debug, Error)]
pub enum Interrupt {
    #[error("integer_ovf")]
    IntegerOverflow,

    #[error("interrupt {0}")]
    Interrupt(usize),

    #[error("dbg_breakpoint {0}")]
    DebugBreakpoint(usize),

    #[error("sys_call {0}")]
    SystemCall(usize),

    #[error("exit {0}")]
    Exit(usize),
}