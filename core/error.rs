use thiserror::Error;

use crate::debug::WatchPoint;

#[derive(Debug, Error, Clone)]
pub enum MmuError {
    #[error("Page not mapped: {0:016x}")]
    PageNotMapped(u64),

    #[error("Page not exist: {0:016x}")]
    PageNotExist(u64),

    #[error("Page already mapped: {0:016x}")]
    PageAlreadyMapped(u64),

    #[error("Access violation: {0:016x}")]
    AccessViolation(u64),

    #[error("Page fault: {0:016x}")]
    PageFault(u64),

    #[error("Fail to write size: {0:016x}")]
    WriteFail(usize),

    #[error("Fail to read size: {0:016x}")]
    ReadFail(usize),
}

#[derive(Debug, Error, Clone)]
pub enum CompileError {}

#[derive(Debug, Error, Clone)]
pub enum CodegenError {
    #[error("Invalid type")]
    InvalidType,
}

#[derive(Debug, Error, Clone)]
pub enum DebugError {
    #[error("MMU error: {0}")]
    MMU(#[from] MmuError),

    #[error("Register is not Exist: {0}")]
    InvalidRegName(String),

    #[error("Watchpoint already exist: {0:?}")]
    WatchpointAlreadyExist(WatchPoint),

    #[error("Watchpoint not exist: {0:?}")]
    WatchpointNotExist(WatchPoint),

    #[error("Breakpoint already exist: {0}")]
    BreakpointAlreadyExist(u64),

    #[error("Breakpoint not exist: {0}")]
    BreakpointNotExist(u64),
}

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("MMU error: {0}")]
    MMU(#[from] MmuError),

    #[error("Compile error: {0}")]
    Compile(#[from] CompileError),

    #[error("Codegen error: {0}")]
    Codegen(#[from] CodegenError),
}
