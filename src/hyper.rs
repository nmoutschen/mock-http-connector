//! Hyper re-exports for version 0.14 and 1.x

#[cfg(all(feature = "hyper_0_14", feature = "hyper_1"))]
compile_error!("must select exactly one feature between hyper_0_14 and hyper_1");

#[cfg(not(any(feature = "hyper_0_14", feature = "hyper_1")))]
compile_error!("must select exactly one feature between hyper_0_14 and hyper_1");

#[cfg(feature = "hyper_0_14")]
pub(crate) use ::hyper_0_14::{
    client::connect::{Connected, Connection},
    header, http, Error, HeaderMap, Method, Request, Response, StatusCode, Uri,
};

#[cfg(feature = "hyper_1")]
pub(crate) use ::hyper_1::{
    header, http, Error, HeaderMap, Method, Request, Response, StatusCode, Uri,
};

#[cfg(feature = "hyper_1")]
pub(crate) use hyper_util::client::legacy::connect::{Connected, Connection};
