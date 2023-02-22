use crate::{error::BoxError, Error};
use hyper::{header::IntoHeaderName, http::HeaderValue, HeaderMap, Method, Request, Uri};
use std::error::Error as StdError;

pub trait With {
    fn with(&self, req: &Request<String>) -> Result<bool, BoxError>;
}

#[derive(Debug)]
pub struct DefaultWith;

impl With for DefaultWith {
    fn with(&self, _req: &Request<String>) -> Result<bool, BoxError> {
        Ok(true)
    }
}

impl<F, E> With for F
where
    for<'r> F: Fn(&'r Request<String>) -> Result<bool, E>,
    E: StdError + Send + Sync + 'static,
{
    fn with(&self, req: &Request<String>) -> Result<bool, BoxError> {
        (self)(req).map_err(Into::into)
    }
}

#[derive(Default, Debug)]
pub struct WithHandler {
    uri: Option<Uri>,
    method: Option<Method>,
    headers: Option<HeaderMap<HeaderValue>>,
    body: Option<Body>,
}

impl WithHandler {
    pub fn with_uri<U>(mut self, uri: U) -> Result<Self, Error>
    where
        U: TryInto<Uri>,
        U::Error: Into<hyper::http::Error>,
    {
        self.uri = Some(uri.try_into().map_err(Into::into)?);
        Ok(self)
    }

    pub fn with_method<M>(mut self, method: M) -> Result<Self, Error>
    where
        M: TryInto<Method>,
        M::Error: Into<hyper::http::Error>,
    {
        self.method = Some(method.try_into().map_err(Into::into)?);
        Ok(self)
    }

    pub fn with_header<K, V>(mut self, key: K, value: V) -> Result<Self, Error>
    where
        K: IntoHeaderName,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        match &mut self.headers {
            Some(headers) => {
                headers.insert(key, value.try_into().map_err(Into::into)?);
            }
            maybe_headers @ None => {
                let mut headers = HeaderMap::new();
                headers.insert(key, value.try_into().map_err(Into::into)?);
                *maybe_headers = Some(headers);
            }
        }

        Ok(self)
    }

    pub fn with_body<B>(mut self, body: B) -> Self
    where
        B: ToString,
    {
        self.body = Some(Body::String(body.to_string()));
        self
    }

    #[cfg(feature = "json")]
    pub fn with_json<V>(mut self, value: V) -> Result<Self, Error>
    where
        V: serde::Serialize,
    {
        self.body = Some(Body::Json(serde_json::to_value(value)?));
        Ok(self)
    }
}

impl With for WithHandler {
    fn with(&self, req: &Request<String>) -> Result<bool, BoxError> {
        if self.uri.is_some() && Some(req.uri()) != self.uri.as_ref() {
            return Ok(false);
        }

        if let Some(headers) = &self.headers {
            for (key, value) in headers {
                if req
                    .headers()
                    .get(key)
                    .map(|rv| value != rv)
                    .unwrap_or(false)
                {
                    return Ok(false);
                }
            }
        }

        match &self.body {
            Some(Body::String(body)) => {
                if body != req.body() {
                    return Ok(false);
                }
            }
            Some(Body::Json(body)) => {
                let payload: serde_json::Value = serde_json::from_str(req.body())?;

                if body != &payload {
                    return Ok(false);
                }
            }
            None => (),
        }

        Ok(true)
    }
}

#[derive(Debug)]
pub enum Body {
    String(String),
    #[cfg(feature = "json")]
    Json(serde_json::Value),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use speculoos::prelude::*;

    #[rstest]
    #[case("http://hello.example/")]
    #[case("http://hello.example/abc")]
    fn with_handler_uri<U>(#[case] uri: U)
    where
        U: TryInto<Uri>,
        U::Error: Into<hyper::http::Error>,
    {
        let with = WithHandler::default();
        assert_that!(with.with_uri(uri)).is_ok();
    }

    #[rstest]
    #[case("GET")]
    #[case(Method::GET)]
    fn with_handler_method<M>(#[case] method: M)
    where
        M: TryInto<Method>,
        M::Error: Into<hyper::http::Error>,
    {
        let with = WithHandler::default();
        assert_that!(with.with_method(method)).is_ok();
    }

    #[rstest]
    #[case("authorization", "Bearer 1234")]
    fn with_handler_header<K, V>(#[case] key: K, #[case] value: V)
    where
        K: IntoHeaderName,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        let with = WithHandler::default();
        assert_that!(with.with_header(key, value)).is_ok();
    }
}
