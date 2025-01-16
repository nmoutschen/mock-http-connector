use crate::hyper::Response;
use crate::{error::BoxError, IntoResponse};
use std::{future::Future, pin::Pin};

pub type ResponseFuture =
    Pin<Box<dyn Future<Output = Result<Response<String>, BoxError>> + Send + Sync + 'static>>;

/// Trait for [`Future`]s that return a valid response for [`crate::Returning`]
///
/// See [`IntoResponse`] for supported return types.
///
/// ## Example
///
/// ```rust
/// # use mock_http_connector::IntoResponseFuture;
/// let fut = async { "hello" };
/// let res_fut = fut.into_response_future();
/// ```
pub trait IntoResponseFuture {
    /// Return a [`Future`] that resolves to `Result<Response<String>, BoxError>`
    fn into_response_future(self) -> ResponseFuture;
}

impl<F> IntoResponseFuture for F
where
    F: Future + Send + Sync + 'static,
    F::Output: IntoResponse,
{
    fn into_response_future(self) -> ResponseFuture {
        Box::pin(async { self.await.into_response() })
    }
}
