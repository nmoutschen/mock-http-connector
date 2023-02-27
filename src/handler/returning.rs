use crate::{error::BoxError, response::ResponseFuture, IntoResponseFuture};
use hyper::{Request, Response, StatusCode};
use std::{borrow::Cow, convert::Infallible, error::Error as StdError};

/// Trait for responses matching mock cases
pub trait Returning: Send + Sync + Sealed {
    /// Return a [`Response`] based on the incoming [`Request`]
    fn returning(&self, req: Request<String>) -> ResponseFuture;
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
            fn returning(&self, _req: ::hyper::Request<String>) -> ResponseFuture {
                #[allow(clippy::ptr_arg)]
                fn response<$($lt),+>(s: &$type) -> Result<Response<String>, BoxError> {
                    let mut res = ::hyper::Response::builder();

                    for (k, v) in ($headers)(s)?.iter() {
                        res = res.header(k, v);
                    }

                    Ok(res
                        .status(($status)(s)?)
                        .body(($body)(s)?)?)
                }

                let res: Result<Response<String>, BoxError> = response(self);
                Box::pin(async move {
                    res
                })
            }
        }

        impl<$($lt),+> Sealed for $type {}
    };
    ($type:ty, $body:expr, $status:expr, $headers:expr) => {
        impl Returning for $type {
            #[allow(clippy::redundant_closure_call)]
            fn returning(&self, _req: ::hyper::Request<String>) -> ResponseFuture {
                fn response(s: &$type) -> Result<Response<String>, BoxError> {
                    let mut res = ::hyper::Response::builder();

                    for (k, v) in ($headers)(s)?.iter() {
                        res = res.header(k, v);
                    }

                    Ok(res
                        .status(($status)(s)?)
                        .body(($body)(s)?)?)
                }

                let res: Result<Response<String>, BoxError> = response(self);
                Box::pin(async move {
                    res
                })
            }
        }

        impl Sealed for $type {}
    };
}

returning!(&'a str, |v: &&str| { Ok::<_, Infallible>(v.to_string()) }, 'a);
returning!(String, |v: &String| { Ok::<_, Infallible>(v.to_string()) });
returning!(Cow<'a, str>, |v: &Cow<'a, str>| { Ok::<_, Infallible>(v.to_string()) }, 'a);
returning!(
    StatusCode,
    |_| Ok::<_, Infallible>(String::new()),
    |v: &StatusCode| Ok::<_, Infallible>(*v)
);
returning!(u16, |_| Ok::<_, Infallible>(String::new()), |v: &u16| {
    StatusCode::try_from(*v)
});

impl<S, B> Returning for (S, B)
where
    (S, B): Send + Sync,
    S: TryInto<StatusCode> + Clone,
    S::Error: StdError + Send + Sync + 'static,
    B: ToString + 'static,
{
    fn returning(&self, _req: Request<String>) -> ResponseFuture {
        let status = self.0.clone().try_into();
        let body = self.1.to_string();
        Box::pin(async { Ok(Response::builder().status(status?).body(body)?) })
    }
}

impl<S, B> Sealed for (S, B)
where
    S: TryInto<StatusCode> + Clone,
    S::Error: StdError + Send + Sync + 'static,
    B: ToString,
{
}

impl<F, R> Returning for F
where
    F: Fn(Request<String>) -> R + Send + Sync,
    R: IntoResponseFuture,
{
    fn returning(&self, req: Request<String>) -> ResponseFuture {
        (self)(req).into_response_future()
    }
}

impl<F, R> Sealed for F
where
    F: Fn(Request<String>) -> R,
    R: IntoResponseFuture,
{
}
