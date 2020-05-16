use std::io::Error as IoError;

use prost::{DecodeError, EncodeError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("empty object `{0}`")]
    EmptyObject(String),
    #[error("invalid ident `{0}`")]
    InvalidIdent(String),
    #[error(transparent)]
    IoError(#[from] IoError),
    #[error(transparent)]
    ProstEncodeError(#[from] EncodeError),
    #[error(transparent)]
    ProstDecodeError(#[from] DecodeError),
    #[error("failed to convert `{0}`")]
    TryFromError(String),
}
