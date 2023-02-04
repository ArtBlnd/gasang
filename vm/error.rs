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
pub enum CompileError {

}