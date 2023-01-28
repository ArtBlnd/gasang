use thiserror::Error;

use crate::MMUError;
use crate::{Vm, VmContext};

#[derive(Clone, Debug, Error)]
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

    #[error("Page Fault")]
    PageFault(#[from] MMUError),

    #[error("Need recompile")]
    NeedRecompile,
}

pub trait InterruptModel {
    unsafe fn interrupt(&self, int: Interrupt, vm: &mut Vm, vm_ctx: &VmContext);
}
