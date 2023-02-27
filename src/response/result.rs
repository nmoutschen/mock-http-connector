use crate::{error::BoxError, IntoResponse};
use hyper::{Response, StatusCode};
use std::error::Error as StdError;

/// Trait for values that can be transformed into `Result<Response<String>, BoxError>`
///
/// All implementations of this trait can be used as the return type for the future passed to
/// [`crate::CaseBuilder::returning`].
///
/// ## Examples
///
/// ### String-types
///
/// String-types will use their value as the response body, with a status code of `200`.
///
/// ```rust
/// # use mock_http_connector::IntoResultResponse;
/// // &str
/// let some_str = "some_str";
/// let res = some_str.into_result_response();
///
/// // Static
/// let some_string = "some_string".to_string();
/// let res = some_string.into_result_response();
/// ```
///
/// ### Status and string-types
///
/// You can pass a status code by passing a value that implements `TryInto<StatusCode>`.
///
/// ```rust
/// # use mock_http_connector::IntoResultResponse;
/// let status = 400;
/// let body = "FILE NOT FOUND";
/// let res = (status, body).into_result_response();
/// ```
///
#[cfg_attr(
    feature = "json",
    doc = r##"
### JSON payloads

This is only supported when the `json` feature flag is set.

```rust
# use mock_http_connector::IntoResultResponse;
# use serde_json::json;
let payload = json!({ "message": "some response" });
let res = payload.into_result_response();
```
"##
)]
pub trait IntoResultResponse {
    /// Transforms self into a `Result<Response<String>, BoxError>`
    fn into_result_response(self) -> Result<Response<String>, BoxError>;
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

impl<S, B> IntoResultResponse for (S, B)
where
    S: TryInto<StatusCode> + 'static,
    S::Error: StdError + Send + Sync + 'static,
    B: ToString + 'static,
{
    fn into_result_response(self) -> Result<Response<String>, BoxError> {
        let status = self.0.try_into();
        let body = self.1.to_string();
        Ok(Response::builder().status(status?).body(body)?)
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
