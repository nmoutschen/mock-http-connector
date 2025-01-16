//! Hyper re-exports for version 0.14 and 1.x

#[cfg(feature = "hyper_0_14")]
pub(crate) use ::hyper_0_14::{
    client::connect::{Connected, Connection},
    header, http,
    service::Service,
    Error, Method, Uri,
};

#[cfg(feature = "hyper_0_14")]
pub use ::hyper_0_14::{Body, Client, HeaderMap, Request, Response, StatusCode};
