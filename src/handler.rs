use std::error::Error as StdError;

use hyper::{Request, Response, StatusCode};

use crate::error::BoxError;

pub struct DefaultErrorHandler;

pub trait ErrorHandler {
    fn handle_error(&self) -> Response<String>;
}

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self) -> Response<String> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("File Not Found".to_string())
            .unwrap()
    }
}

impl<F, R> ErrorHandler for F
where
    F: Fn() -> R,
    R: IntoResponse,
{
    fn handle_error(&self) -> Response<String> {
        (self)().into_response()
    }
}

pub trait Returning {
    fn returning(&self, req: Request<String>) -> Result<Response<String>, BoxError>;
}

impl<F, R, E> Returning for F
where
    F: Fn(Request<String>) -> Result<R, E>,
    R: IntoResponse,
    E: StdError + Send + Sync + 'static,
{
    fn returning(&self, req: Request<String>) -> Result<Response<String>, BoxError> {
        Ok((self)(req).map(|r| r.into_response())?)
    }
}

pub trait IntoResponse {
    fn into_response(self) -> Response<String>;
}

impl<B> IntoResponse for Response<B>
where
    B: ToString,
{
    fn into_response(self) -> Response<String> {
        self.map(|b| b.to_string())
    }
}

pub trait With {
    fn with(&self, req: &Request<String>) -> Result<bool, BoxError>;
}

pub struct DefaultWith;

impl With for DefaultWith {
    fn with(&self, _req: &Request<String>) -> Result<bool, BoxError> {
        Ok(true)
    }
}

impl<F, E> With for F
where
    for<'r> F: Fn(&'r Request<String>) -> Result<bool, E>,
    E: StdError + Send + Sync + 'static,
{
    fn with(&self, req: &Request<String>) -> Result<bool, BoxError> {
        (self)(req).map_err(Into::into)
    }
}
