use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum MMUError {
    #[error("Page not mapped")]
    PageNotMapped,

    #[error("Page already mapped")]
    PageAlreadyMapped,

    #[error("Access violation")]
    AccessViolation,

    #[error("Page fault")]
    PageFault,
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
