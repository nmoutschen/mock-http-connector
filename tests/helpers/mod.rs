#![allow(unused)]

#[cfg(feature = "hyper_0_14")]
pub use hyper_0_14::{header::HeaderName, http, Method, Request, Response, StatusCode};

#[cfg(feature = "hyper_1")]
pub use hyper_1::{header::HeaderName, http, Method, Request, Response, StatusCode};

#[cfg(feature = "hyper_0_14")]
pub fn client<C>(connector: C) -> hyper_0_14::Client<C>
where
    C: hyper_0_14::client::connect::Connect + Clone,
{
    hyper_0_14::Client::builder().build(connector)
}

#[cfg(feature = "hyper_1")]
pub fn client<C>(
    connector: C,
) -> hyper_util::client::legacy::Client<C, http_body_util::Full<hyper_1::body::Bytes>>
where
    C: hyper_util::client::legacy::connect::Connect + Clone,
{
    hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(connector)
}

#[cfg(feature = "hyper_0_14")]
pub async fn to_bytes<T>(body: T) -> hyper_0_14::body::Bytes
where
    T: hyper_0_14::body::HttpBody,
    T::Error: std::error::Error,
{
    hyper_0_14::body::to_bytes(body).await.unwrap()
}

#[cfg(feature = "hyper_1")]
pub async fn to_bytes<T>(body: T) -> hyper_1::body::Bytes
where
    T: hyper_1::body::Body,
    T::Error: std::error::Error,
{
    use http_body_util::BodyExt;
    body.collect().await.unwrap().to_bytes()
}
