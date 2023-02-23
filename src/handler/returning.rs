use super::IntoResultResponse;
use crate::error::BoxError;
use hyper::{Request, Response, StatusCode};
use std::{borrow::Cow, convert::Infallible, error::Error as StdError};

/// Trait for responses matching mock cases
pub trait Returning: Send + Sync + Sealed {
    /// Return a [`Response`] based on the incoming [`Request`]
    fn returning(&self, req: Request<String>) -> Result<Response<String>, BoxError>;
}

/// Sealed trait to avoid additional implementations of [`Returning`]
pub trait Sealed {}

macro_rules! returning {
    // With no status
    ($type:ty, $body:expr, $($lt:lifetime),+) => {
        returning!($type, $body, |_| Ok::<_, ::std::convert::Infallible>(::hyper::StatusCode::OK), |_| Ok::<_, ::std::convert::Infallible>(::hyper::HeaderMap::new()), $($lt),+);
    };
    ($type:ty, $body:expr) => {
        returning!($type, $body, |_| Ok::<_, ::std::convert::Infallible>(::hyper::StatusCode::OK), |_| Ok::<_, ::std::convert::Infallible>(::hyper::HeaderMap::new()));
    };

    // With no headers
    ($type:ty, $body:expr, $status:expr, $($lt:lifetime),+) => {
        returning!($type, $body, $status, |_| Ok::<_, ::std::convert::Infallible>(::hyper::HeaderMap::new()), $($lt),+);
    };
    ($type:ty, $body:expr, $status:expr) => {
        returning!($type, $body, $status, |_| Ok::<_, ::std::convert::Infallible>(::hyper::HeaderMap::new()));
    };

    ($type:ty, $body:expr, $status:expr, $headers:expr, $($lt:lifetime),+) => {
        impl<$($lt),+> Returning for $type {
            #[allow(clippy::redundant_closure_call)]
            fn returning(&self, _req: ::hyper::Request<String>) -> Result<Response<String>, crate::error::BoxError> {
                let mut res = ::hyper::Response::builder();

                for (k, v) in ($headers)(self)?.iter() {
                    res = res.header(k, v);
                }

                Ok(res
                    .status(($status)(self)?)
                    .body(($body)(self)?)?)
            }
        }

        impl<$($lt),+> Sealed for $type {}
    };
    ($type:ty, $body:expr, $status:expr, $headers:expr) => {
        impl Returning for $type {
            #[allow(clippy::redundant_closure_call)]
            fn returning(&self, _req: ::hyper::Request<String>) -> Result<::hyper::Response<String>, crate::error::BoxError> {
                let mut res = ::hyper::Response::builder();

                for (k, v) in ($headers)(self)?.iter() {
                    res = res.header(k, v);
                }

                Ok(res
                    .status(($status)(self)?)
                    .body(($body)(self)?)?)
            }
        }

        impl Sealed for $type {}
    };
}

returning!(&'a str, |v: &Self| { Ok::<_, Infallible>(v.to_string()) }, 'a);
returning!(String, |v: &Self| { Ok::<_, Infallible>(v.to_string()) });
returning!(Cow<'a, str>, |v: &Self| { Ok::<_, Infallible>(v.to_string()) }, 'a);
returning!(
    StatusCode,
    |_| Ok::<_, Infallible>(String::new()),
    |v: &Self| Ok::<_, Infallible>(*v)
);
returning!(u16, |_| Ok::<_, Infallible>(String::new()), |v: &Self| {
    StatusCode::try_from(*v)
});

impl<F, R> Returning for F
where
    F: Fn(Request<String>) -> R + Send + Sync,
    R: IntoResultResponse,
{
    fn returning(&self, req: Request<String>) -> Result<Response<String>, BoxError> {
        (self)(req).into_result_response()
    }
}

impl<S, B> Returning for (S, B)
where
    (S, B): Send + Sync,
    S: TryInto<StatusCode> + Clone,
    S::Error: StdError + Send + Sync + 'static,
    B: ToString,
{
    fn returning(&self, _req: Request<String>) -> Result<Response<String>, BoxError> {
        Ok(Response::builder()
            .status(self.0.clone().try_into()?)
            .body(self.1.to_string())?)
    }
}

impl<F, R> Sealed for F
where
    F: Fn(Request<String>) -> R,
    R: IntoResultResponse,
{
}

impl<S, B> Sealed for (S, B)
where
    S: TryInto<StatusCode> + Clone,
    S::Error: StdError + Send + Sync + 'static,
    B: ToString,
{
}
