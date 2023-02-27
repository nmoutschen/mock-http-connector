use hyper::{Response, StatusCode};

mod future;
mod result;
pub use future::{IntoResponseFuture, ResponseFuture};
pub use result::IntoResultResponse;

/// Trait for values that can be transformed into `Response<String>`
///
/// All implementations of this trait can be used for [`crate::Builder::missing`] and
/// [`crate::Builder::error`].
///
/// ## Examples
///
/// ### Other [`hyper::Request`]s
///
/// ```rust
/// # use mock_http_connector::IntoResponse;
/// # use hyper::Response;
/// let res = Response::builder().status(200).body("hello").unwrap();
/// let res = res.into_response();
/// ```
///
/// ### [`StatusCode`] and string-type
///
/// ```rust
/// # use mock_http_connector::IntoResponse;
/// # use hyper::StatusCode;
/// let status = StatusCode::NOT_FOUND;
/// let body = "FILE NOT FOUND";
/// let res = (status, body).into_response();
/// ```
pub trait IntoResponse {
    /// Transforms self into a `Response<String>`
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
