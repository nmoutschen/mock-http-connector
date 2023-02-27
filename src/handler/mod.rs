mod returning;
mod with;

use hyper::{Response, StatusCode};
pub use returning::Returning;

pub use with::{DefaultWith, With, WithHandler};

use crate::IntoResponse;

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
