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
    TryFromError(
        String,
        #[source] Box<dyn ::std::error::Error + Sync + Send + 'static>,
    ),
}

impl Error {
    pub fn new_empty_object<T: ToString>(ident: T) -> Self {
        Self::EmptyObject(ident.to_string())
    }

    pub fn new_invalid_ident<T: ToString>(ident: T) -> Self {
        Self::InvalidIdent(ident.to_string())
    }

    pub fn new_try_from_error<
        T: ToString,
        E: Into<Box<dyn ::std::error::Error + Sync + Send + 'static>>,
    >(
        ident: T,
        e: E,
    ) -> Self {
        Self::TryFromError(ident.to_string(), e.into())
    }
}
