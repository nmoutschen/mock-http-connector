//! Hyper re-exports for version 0.14 and 1.x

#[cfg(feature = "hyper_0_14")]
pub(crate) use ::hyper_0_14::{
    client::connect::{Connected, Connection},
    header, http, Error, Method, Uri,
};

#[cfg(feature = "hyper_0_14")]
pub use ::hyper_0_14::{
    Body, Builder, Client, HeaderMap, HttpBody, Method, Request, Response, StatusCode,
};

#[cfg(feature = "hyper_1")]
pub(crate) use ::hyper_1::{header, Error, Uri};

#[cfg(feature = "hyper_1")]
pub use hyper_util::client::legacy::connect::{Connected, Connection};

#[cfg(feature = "hyper_1")]
pub use ::hyper_1::{
    body::{Body as HttpBody, Bytes},
    http, HeaderMap, Method, Request, Response, StatusCode,
};

#[cfg(feature = "hyper_1")]
pub use ::hyper_util::client::legacy::{Builder, Client};

#[cfg(feature = "hyper_1")]
pub type Body = ::http_body_util::Full<hyper_1::body::Bytes>;

#[cfg(feature = "hyper_0_14")]
pub fn client_builder() -> Builder {
    Client::builder()
}

#[cfg(feature = "hyper_1")]
pub fn client_builder() -> Builder {
    Client::builder(hyper_util::rt::TokioExecutor::new())
}

#[cfg(feature = "hyper_1")]
pub async fn to_bytes<T>(body: T) -> Bytes
where
    T: hyper_1::body::Body,
    T::Error: std::error::Error,
{
    use http_body_util::BodyExt;
    body.collect().await.unwrap().to_bytes()
}
