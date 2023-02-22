mod returning;
mod with;

use hyper::{Response, StatusCode};
pub use returning::Returning;
use std::error::Error as StdError;
pub use with::{DefaultWith, With, WithHandler};

use crate::error::BoxError;

pub trait DefaultHandler {
    fn handle(&self) -> Response<String>;
}

pub struct DefaultErrorHandler;

impl DefaultHandler for DefaultErrorHandler {
    fn handle(&self) -> Response<String> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("".to_string())
            .unwrap()
    }
}

pub struct DefaultMissingHandler;

impl DefaultHandler for DefaultMissingHandler {
    fn handle(&self) -> Response<String> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("".to_string())
            .unwrap()
    }
}

impl<F, R> DefaultHandler for F
where
    F: Fn() -> R,
    R: IntoResponse,
{
    fn handle(&self) -> Response<String> {
        (self)().into_response()
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

pub trait IntoResultResponse {
    fn into_result_response(self) -> Result<Response<String>, BoxError>;
}

impl<R> IntoResultResponse for R
where
    R: IntoResponse,
{
    fn into_result_response(self) -> Result<Response<String>, BoxError> {
        Ok(self.into_response())
    }
}

impl<R, E> IntoResultResponse for Result<R, E>
where
    R: IntoResponse,
    E: StdError + Send + Sync + 'static,
{
    fn into_result_response(self) -> Result<Response<String>, BoxError> {
        Ok(self.map(|r| r.into_response())?)
    }
}

impl<'s> IntoResultResponse for &'s str {
    fn into_result_response(self) -> Result<Response<String>, BoxError> {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(self.to_string())?)
    }
}

impl IntoResultResponse for String {
    fn into_result_response(self) -> Result<Response<String>, BoxError> {
        Ok(Response::builder().status(StatusCode::OK).body(self)?)
    }
}

#[cfg(feature = "json")]
impl IntoResultResponse for serde_json::Value {
    fn into_result_response(self) -> Result<Response<String>, BoxError> {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&self)?)?)
    }
}

impl<S, B> IntoResponse for (S, B)
where
    S: Into<StatusCode>,
    B: ToString,
{
    fn into_response(self) -> Response<String> {
        Response::builder()
            .status(self.0)
            .body(self.1.to_string())
            .unwrap()
    }
}
