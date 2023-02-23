use crate::{
    case::Case,
    handler::{
        DefaultErrorHandler, DefaultMissingHandler, DefaultWith, Returning, With, WithHandler,
    },
    Connector, Error,
};
use hyper::{header::IntoHeaderName, http::HeaderValue, Method, Request, Response, Uri};
use std::error::Error as StdError;

/// Builder for [`Connector`]
pub struct Builder<FE = DefaultErrorHandler, FM = DefaultMissingHandler> {
    cases: Vec<Case>,
    error_handler: FE,
    missing_handler: FM,
}

impl<FE, FM> Builder<FE, FM> {
    /// Create a new expectation
    pub fn expect(&mut self) -> CaseBuilder<'_, FE, FM> {
        CaseBuilder::new(self)
    }

    /// Remplace the default error handler
    ///
    /// `error_handler` should be a function or closure that returns a `hyper::Response<String>`.
    ///
    /// It will be called whenever there is an internal error to return a valid payload to the
    /// client.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use hyper::Response;
    /// # use mock_http_connector::Connector;
    /// let builder = Connector::builder().error(|| Response::builder().body("Something went wrong!".to_string()).unwrap());
    /// ```
    pub fn error<NF>(self, error_handler: NF) -> Builder<NF, FM>
    where
        NF: Fn() -> Response<String>,
    {
        Builder {
            cases: self.cases,
            error_handler,
            missing_handler: self.missing_handler,
        }
    }

    /// Remplace the default missing handler
    ///
    /// `missing_handler` should be a function or closure that returns a `hyper::Response<String>`.
    ///
    /// It will be called whenever no expectation matches the incoming request.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use hyper::Response;
    /// # use mock_http_connector::Connector;
    /// let builder = Connector::builder().missing(|| Response::builder().body("Request not found!".to_string()).unwrap());
    /// ```
    pub fn missing<NF>(self, missing_handler: NF) -> Builder<FE, NF>
    where
        NF: Fn() -> Response<String>,
    {
        Builder {
            cases: self.cases,
            error_handler: self.error_handler,
            missing_handler,
        }
    }

    /// Build the [`Connector`]
    ///
    /// This will consume the [`Builder`]
    pub fn build(self) -> Connector<FE, FM> {
        Connector::new(self.cases, self.error_handler, self.missing_handler)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            cases: Default::default(),
            error_handler: DefaultErrorHandler,
            missing_handler: DefaultMissingHandler,
        }
    }
}

/// Builder for specific mock cases
pub struct CaseBuilder<'b, FE, FM, W = DefaultWith> {
    builder: &'b mut Builder<FE, FM>,
    with: W,
    count: Option<usize>,
}

impl<'b, FE, FM> CaseBuilder<'b, FE, FM> {
    fn new(builder: &'b mut Builder<FE, FM>) -> Self {
        Self {
            builder,
            with: DefaultWith,
            count: None,
        }
    }

    /// Pass a function or closure to check if the incoming payload matches this mock case
    ///
    /// If you only need to validate the [`Uri`], [`Method`], headers, or incoming payload, you
    /// should use one of the other `with_*` methods. You also cannot combine this validator with
    /// the other `with` methods.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use hyper::{Response, Request};
    /// # use mock_http_connector::Connector;
    /// # use std::convert::Infallible;
    /// let mut builder = Connector::builder();
    /// builder
    ///     .expect()
    ///     .with(|req: &Request<String>| Ok::<_, Infallible>(req.body().contains("hello")))
    ///     .returning("OK");
    /// ```
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with<W, E>(self, with: W) -> CaseBuilder<'b, FE, FM, W>
    where
        for<'r> W: Fn(&'r Request<String>) -> Result<bool, E>,
        E: StdError + Send + Sync + 'static,
    {
        CaseBuilder {
            builder: self.builder,
            with,
            count: self.count,
        }
    }

    /// Match requests with the specified [`Uri`]
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use hyper::Response;
    /// # use mock_http_connector::{Connector, Error};
    /// # || {
    /// let mut builder = Connector::builder();
    /// builder
    ///     .expect()
    ///     .with_uri("https://example.test/hello")?
    ///     .returning("OK");
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_header`, but not with `with`.
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_uri<U>(self, uri: U) -> Result<CaseBuilder<'b, FE, FM, WithHandler>, Error>
    where
        U: TryInto<Uri>,
        U::Error: Into<hyper::http::Error>,
    {
        Ok(CaseBuilder {
            builder: self.builder,
            with: WithHandler::default().with_uri(uri)?,
            count: self.count,
        })
    }

    /// Match requests with the specified [`Method`]
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use hyper::Response;
    /// # use mock_http_connector::{Connector, Error};
    /// # || {
    /// let mut builder = Connector::builder();
    /// builder
    ///     .expect()
    ///     .with_method("GET")?
    ///     .returning("OK");
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_method<M>(self, method: M) -> Result<CaseBuilder<'b, FE, FM, WithHandler>, Error>
    where
        M: TryInto<Method>,
        M::Error: Into<hyper::http::Error>,
    {
        Ok(CaseBuilder {
            builder: self.builder,
            with: WithHandler::default().with_method(method)?,
            count: self.count,
        })
    }

    /// Match requests that contains the specific header
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use hyper::Response;
    /// # use mock_http_connector::{Connector, Error};
    /// # || {
    /// let mut builder = Connector::builder();
    /// builder
    ///     .expect()
    ///     .with_header("content-type", "application/json")?
    ///     .returning("OK");
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_header<K, V>(
        self,
        key: K,
        value: V,
    ) -> Result<CaseBuilder<'b, FE, FM, WithHandler>, Error>
    where
        K: IntoHeaderName,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        Ok(CaseBuilder {
            builder: self.builder,
            with: WithHandler::default().with_header(key, value)?,
            count: self.count,
        })
    }

    /// Match requests that contains the provided payload
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use hyper::Response;
    /// # use mock_http_connector::Connector;
    /// let mut builder = Connector::builder();
    /// builder
    ///     .expect()
    ///     .with_body("some body")
    ///     .returning("OK");
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    ///
    /// A mock case only supports `with_body`, `with_json`, or `with_json_value`, but not multiple
    /// ones at the same time.
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_body<B>(self, body: B) -> CaseBuilder<'b, FE, FM, WithHandler>
    where
        B: ToString,
    {
        CaseBuilder {
            builder: self.builder,
            with: WithHandler::default().with_body(body),
            count: self.count,
        }
    }

    /// Match requests with a body that exactly matches the provided JSON payload
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use hyper::Response;
    /// # use mock_http_connector::{Connector, Error};
    /// # || {
    /// let mut builder = Connector::builder();
    /// builder
    ///     .expect()
    ///     .with_json(serde_json::json!({"status": "OK"}))?
    ///     .returning("OK");
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    ///
    /// A mock case only supports `with_body`, `with_json`, or `with_json_value`, but not multiple
    /// ones at the same time.
    #[cfg(feature = "json")]
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_json<V>(self, value: V) -> Result<CaseBuilder<'b, FE, FM, WithHandler>, Error>
    where
        V: serde::Serialize,
    {
        Ok(CaseBuilder {
            builder: self.builder,
            with: WithHandler::default().with_json(value)?,
            count: self.count,
        })
    }

    /// Match requests that contains the provided JSON payload, but may contain other properties
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    pub fn with_json_partial<V>(
        self,
        value: V,
    ) -> Result<CaseBuilder<'b, FE, FM, WithHandler>, Error>
    where
        V: serde::Serialize,
    {
        Ok(CaseBuilder {
            builder: self.builder,
            with: WithHandler::default().with_json_partial(value)?,
            count: self.count,
        })
    }
}

impl<'b, FE, FM> CaseBuilder<'b, FE, FM, WithHandler> {
    #[doc(hidden)]
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_uri<U>(mut self, uri: U) -> Result<Self, Error>
    where
        U: TryInto<Uri>,
        U::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.with_uri(uri)?;
        Ok(self)
    }

    #[doc(hidden)]
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_method<M>(mut self, method: M) -> Result<Self, Error>
    where
        M: TryInto<Method>,
        M::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.with_method(method)?;
        Ok(self)
    }

    #[doc(hidden)]
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_header<K, V>(mut self, key: K, value: V) -> Result<Self, Error>
    where
        K: IntoHeaderName,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.with_header(key, value)?;
        Ok(self)
    }

    #[doc(hidden)]
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_body<B>(mut self, body: B) -> Self
    where
        B: ToString,
    {
        self.with = self.with.with_body(body);
        self
    }

    #[doc(hidden)]
    #[cfg(feature = "json")]
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_json<V>(mut self, value: V) -> Result<Self, Error>
    where
        V: serde::Serialize,
    {
        self.with = self.with.with_json(value)?;
        Ok(self)
    }

    #[doc(hidden)]
    #[cfg(feature = "json")]
    #[must_use = "this does nothing until you call `returning`"]
    pub fn with_json_partial<V>(mut self, value: V) -> Result<Self, Error>
    where
        V: serde::Serialize,
    {
        self.with = self.with.with_json_partial(value)?;
        Ok(self)
    }
}

impl<'b, FE, FM, W> CaseBuilder<'b, FE, FM, W> {
    /// Mark how many times this mock case can be called
    ///
    /// Nothing enforces how many times a mock case is called, but you can use the `checkpoint`
    /// method on the [`Connector`] to ensure all methods were called the right amount of times.
    #[must_use = "this does nothing until you call `returning`"]
    pub fn times(self, count: usize) -> Self {
        Self {
            count: Some(count),
            ..self
        }
    }
}

impl<'b, FE, FM, W> CaseBuilder<'b, FE, FM, W>
where
    W: With + 'static,
{
    /// Mark what will generate the response for a given mock case
    ///
    /// You can either pass a static value, or a function or closure that takes a `Request<String>`
    /// as an input.
    ///
    /// See the documentation for [`Returning`] to see the full list of what is accepted by this
    /// method.
    pub fn returning<R>(self, returning: R)
    where
        R: Returning + 'static,
    {
        let case = Case::new(self.with, returning, self.count);
        self.builder.cases.push(case);
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::*;

    #[test]
    fn test_with() {
        let mut builder = Connector::builder();
        builder
            .expect()
            .with(|req: &Request<String>| Ok::<_, Infallible>(req.body().contains("hello")))
            .returning("OK");
    }
}
