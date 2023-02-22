use super::IntoResultResponse;
use crate::error::BoxError;
use hyper::{Request, Response, StatusCode};
use std::error::Error as StdError;

/// Trait for responses matching mock cases
pub trait Returning: Sealed {
    /// Return a [`Response`] based on the incoming [`Request`]
    fn returning(&self, req: Request<String>) -> Result<Response<String>, BoxError>;
}

impl<F, R> Returning for F
where
    F: Fn(Request<String>) -> R,
    R: IntoResultResponse,
{
    fn returning(&self, req: Request<String>) -> Result<Response<String>, BoxError> {
        (self)(req).into_result_response()
    }
}

impl<'s> Returning for &'s str {
    fn returning(&self, _req: Request<String>) -> Result<Response<String>, BoxError> {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(self.to_string())?)
    }
}

impl<S, B> Returning for (S, B)
where
    S: TryInto<StatusCode> + Clone,
    S::Error: StdError + Send + Sync + 'static,
    B: ToString,
{
    fn returning(&self, _req: Request<String>) -> Result<Response<String>, BoxError> {
        Ok(Response::builder()
            .status(self.0.clone().try_into()?)
            .body(self.1.to_string())?)
    }
}

pub trait Sealed {}

impl<F, R> Sealed for F
where
    F: Fn(Request<String>) -> R,
    R: IntoResultResponse,
{
}

impl<'s> Sealed for &'s str {}

impl<S, B> Sealed for (S, B)
where
    S: TryInto<StatusCode> + Clone,
    S::Error: StdError + Send + Sync + 'static,
    B: ToString,
{
}
