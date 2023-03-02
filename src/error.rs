use std::{error::Error as StdError, sync::PoisonError};

use hyper::Request;
use miette::{Diagnostic, NamedSource, SourceOffset, SourceSpan};

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

    /// [`With`] handler errors
    #[error("with handler error: {0}")]
    With(#[from] WithError),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Self::Lock(value.to_string())
    }
}

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum WithError {
    #[error("invalid body: {0}")]
    Body(BodyError),

    #[error("unknown error: {0}")]
    Unknown(BoxError),
}

impl WithError {
    pub fn body<E>(err: E) -> Self
    where
        E: Into<BodyError>,
    {
        Self::Body(err.into())
    }
}

#[derive(Debug, thiserror::Error, Diagnostic)]
#[error("invalid body: {err}")]
pub struct BodyError {
    #[source_code]
    src: NamedSource,
    #[label]
    body: SourceSpan,
    err: BoxError,
}

#[cfg(feature = "json")]
impl<'b> From<(&'b str, serde_json::Error)> for BodyError {
    fn from((body, err): (&'b str, serde_json::Error)) -> Self {
        let body = body.to_string();
        BodyError {
            src: NamedSource::new("body", body.clone()),
            body: SourceOffset::from_location(body, err.line(), err.column()).into(),
            err: err.into(),
        }
    }
}

pub type BoxError = Box<dyn StdError + Send + Sync>;
