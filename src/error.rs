use std::{error::Error as StdError, sync::PoisonError};

use hyper::Request;

use crate::case::Checkpoint;

/// Errors generated by this crate
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Errors while checking if all mock cases were called the right number of times
    #[error("checkpoint error: {0:?}")]
    Checkpoint(Vec<Checkpoint>),

    /// Error from the [`hyper`] crate
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    /// Error from [`hyper::http`]
    #[error("http error: {0}")]
    Http(#[from] hyper::http::Error),

    /// Error from [`httparse`]
    #[error("httparse error: {0}")]
    Httparse(#[from] httparse::Error),

    /// JSON serialization/deserialization error
    #[cfg(feature = "json")]
    #[error("JSON serde error: {0}")]
    Json(#[from] serde_json::Error),

    /// Mutex lock poisoning
    #[error("lock poison error: {0}")]
    Lock(String),

    /// No match found for the incoming [`Request`]
    #[error("no cases matched the request: {0:?}")]
    NotFound(Request<String>),

    /// Runtime errors
    #[error("transparent")]
    Runtime(#[from] BoxError),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Self::Lock(value.to_string())
    }
}

pub type BoxError = Box<dyn StdError + Send + Sync>;
