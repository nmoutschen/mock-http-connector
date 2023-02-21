use crate::case::CheckpointErrors;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("http error: {0}")]
    Http(#[from] hyper::http::Error),

    #[error("lock poison error: {0}")]
    Lock(String),

    #[error("mismatch in request count for {} cases: {0}", .0.len())]
    Checkpoint(CheckpointErrors),

    #[error("response not found")]
    ResponseNotFound,
}
