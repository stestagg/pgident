use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentError {
    #[error("Null byte in identifier")]
    NullByteError(),
    #[error("Zero length identifier")]
    ZeroLengthError(),
}