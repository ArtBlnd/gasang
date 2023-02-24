use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum MMUError {
    #[error("Page not mapped")]
    PageNotMapped,

    #[error("Page already mapped")]
    PageAlreadyMapped,

    #[error("Access violation: {0:016x}")]
    AccessViolation(u64),

    #[error("Page fault: {0:016x}")]
    PageFault(u64),
}

#[derive(Debug, Error, Clone)]
pub enum CompileError {}

#[derive(Debug, Error, Clone)]
pub enum CodegenError {
    #[error("Invalid type")]
    InvalidType,
}

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("MMU error: {0}")]
    MMU(#[from] MMUError),

    #[error("Compile error: {0}")]
    Compile(#[from] CompileError),

    #[error("Codegen error: {0}")]
    Codegen(#[from] CodegenError),
}
