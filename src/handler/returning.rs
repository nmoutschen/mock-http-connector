use hyper::{Request, Response};

use crate::error::BoxError;

use super::IntoResultResponse;

pub trait Returning {
    fn returning(&self, req: Request<String>) -> Result<Response<String>, BoxError>;
}

impl<F, R> Returning for F
where
    F: Fn(Request<String>) -> R,
    R: IntoResultResponse,
{
    fn returning(&self, req: Request<String>) -> Result<Response<String>, BoxError> {
        (self)(req).into_result_response()
    }
}
