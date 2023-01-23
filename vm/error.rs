use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum MMUError {
    #[error("Page not mapped")]
    PageNotMapped,

    #[error("Page already mapped")]
    PageAlreadyMapped,
}
