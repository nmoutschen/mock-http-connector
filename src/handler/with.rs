use crate::error::BoxError;
use hyper::{header::IntoHeaderName, http::HeaderValue, HeaderMap, Request, Uri};
use std::error::Error as StdError;

pub trait With {
    fn with(&self, req: &Request<String>) -> Result<bool, BoxError>;
}

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
    headers: Option<HeaderMap<HeaderValue>>,
    body: Option<String>,
}

impl WithHandler {
    pub fn with_uri<U>(mut self, uri: U) -> Result<Self, BoxError>
    where
        U: TryInto<Uri>,
        U::Error: StdError + Send + Sync + 'static,
    {
        self.uri = Some(uri.try_into()?);
        Ok(self)
    }

    pub fn with_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: IntoHeaderName,
        V: Into<HeaderValue>,
    {
        match &mut self.headers {
            Some(headers) => {
                headers.insert(key, value.into());
            }
            maybe_headers @ None => {
                let mut headers = HeaderMap::new();
                headers.insert(key, value.into());
                *maybe_headers = Some(headers);
            }
        }

        self
    }

    pub fn with_body<B>(mut self, body: B) -> Self
    where
        B: ToString,
    {
        self.body = Some(body.to_string());
        self
    }

    #[cfg(feature = "json")]
    pub fn with_json<V>(mut self, value: V) -> Result<Self, serde_json::Error>
    where
        V: serde::Serialize,
    {
        self.body = Some(serde_json::to_string(&value)?);
        Ok(self)
    }
}

impl With for WithHandler {
    fn with(&self, req: &Request<String>) -> Result<bool, BoxError> {
        if self.uri.is_some() && Some(req.uri()) != self.uri.as_ref() {
            return Ok(false);
        }

        if self.headers.is_some() && Some(req.headers()) != self.headers.as_ref() {
            return Ok(false);
        }

        if self.body.is_some() && Some(req.body()) != self.body.as_ref() {
            return Ok(false);
        }

        Ok(true)
    }
}
