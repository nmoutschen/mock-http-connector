use std::{error::Error as StdError, sync::PoisonError};

use crate::case::Checkpoint;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("http error: {0}")]
    Http(#[from] hyper::http::Error),

    #[error("lock poison error: {0}")]
    Lock(String),

    #[error("response not found")]
    ResponseNotFound,

    #[error("transparent")]
    Runtime(#[from] BoxError),

    #[error("checkpoint error: {0:?}")]
    Checkpoint(Vec<Checkpoint>),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Self::Lock(value.to_string())
    }
}

pub type BoxError = Box<dyn StdError + Send + Sync>;
