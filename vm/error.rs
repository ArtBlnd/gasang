use thiserror::Error;

#[derive(Debug, Error)]
pub enum MMUError {
    #[error("Page not mapped")]
    PageNotMapped,

    #[error("Page already mapped")]
    PageAlreadyMapped,
}
