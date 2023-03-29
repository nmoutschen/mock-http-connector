use crate::{
    case::Case,
    connector::InnerConnector,
    handler::{DefaultWith, Returning, With, WithHandler},
    Connector, Error, Level, Report,
};
use hyper::{
    http::{HeaderName, HeaderValue},
    Method, Request, Uri,
};
use std::error::Error as StdError;

/// Builder for [`Connector`]
#[derive(Default)]
pub struct Builder {
    inner: InnerConnector,
}

impl Builder {
    /// Build into an usable [`Connector`]
    pub fn build(self) -> Connector {
        Connector::from_inner(self.inner)
    }

    /// Set the diagnostics [`Level`] for the connector
    pub fn level(&mut self, level: Level) {
        self.inner.level = level;
    }

    /// Create a new expected case
    pub fn expect(&mut self) -> CaseBuilder<'_> {
        CaseBuilder::new(&mut self.inner)
    }
}

/// Builder for specific mock cases
///
/// ## Example
///
/// ```rust
/// # use mock_http_connector::Connector;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut builder = Connector::builder();
/// let mut case_builder = builder.expect();
///
/// case_builder
///     .with_uri("https://test.example/some/path")
///     .times(3)
///     .returning("Some response")?;
/// # Ok(())
/// # }
/// ```
#[must_use = "case builders do nothing until you call the `returning` method"]
pub struct CaseBuilder<'c, W = DefaultWith> {
    connector: &'c mut InnerConnector,
    with: Result<W, Error>,
    count: Option<usize>,
}

impl<'c> CaseBuilder<'c> {
    pub(crate) fn new(connector: &'c mut InnerConnector) -> Self {
        Self {
            connector,
            with: Ok(DefaultWith),
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
    /// # use mock_http_connector::{Connector, Error};
    /// # use std::convert::Infallible;
    /// # || {
    /// let mut builder = Connector::builder();
    /// builder
    ///     .expect()
    ///     .with(|req: &Request<String>| Ok::<_, Infallible>(req.body().contains("hello")))
    ///     .returning("OK")?;
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    pub fn with<W, E, R>(self, with: W) -> CaseBuilder<'c, W>
    where
        for<'r> W: Fn(&'r Request<String>) -> Result<R, E>,
        R: Into<Report>,
        E: StdError + Send + Sync + 'static,
    {
        CaseBuilder {
            connector: self.connector,
            with: Ok(with),
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
    ///     .with_uri("https://example.test/hello")
    ///     .returning("OK")?;
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_header`, but not with `with`.
    pub fn with_uri<U>(self, uri: U) -> CaseBuilder<'c, WithHandler>
    where
        U: TryInto<Uri>,
        U::Error: Into<hyper::http::Error>,
    {
        CaseBuilder {
            connector: self.connector,
            with: WithHandler::default().with_uri(uri),
            count: self.count,
        }
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
    ///     .with_method("GET")
    ///     .returning("OK")?;
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    pub fn with_method<M>(self, method: M) -> CaseBuilder<'c, WithHandler>
    where
        M: TryInto<Method>,
        M::Error: Into<hyper::http::Error>,
    {
        CaseBuilder {
            connector: self.connector,
            with: WithHandler::default().with_method(method),
            count: self.count,
        }
    }

    /// Match requests that contains the specific header
    ///
    /// An HTTP request can contain multiple headers with the same key, but different values. This
    /// checks that there is at least one value matching. If you want to ensure that there is only
    /// one entry for this key, consider using `with_header_once`.
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
    ///     .with_header("content-type", "application/json")
    ///     .returning("OK")?;
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    pub fn with_header<K, V>(self, key: K, value: V) -> CaseBuilder<'c, WithHandler>
    where
        K: TryInto<HeaderName>,
        K::Error: Into<hyper::http::Error>,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        CaseBuilder {
            connector: self.connector,
            with: WithHandler::default().with_header(key, value),
            count: self.count,
        }
    }

    /// Match requests that contains the specific header
    ///
    /// An HTTP request can contain multiple headers with the same key, but different values. This
    /// checks that there is only one value for the given header.
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
    ///     .with_header_once("content-type", "application/json")
    ///     .returning("OK")?;
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    pub fn with_header_once<K, V>(self, key: K, value: V) -> CaseBuilder<'c, WithHandler>
    where
        K: TryInto<HeaderName>,
        K::Error: Into<hyper::http::Error>,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        CaseBuilder {
            connector: self.connector,
            with: WithHandler::default().with_header_once(key, value),
            count: self.count,
        }
    }

    /// Match requests that contains the specific header
    ///
    /// An HTTP request can contain multiple headers with the same key, but different values. This
    /// checks that all entries correspond to the given set of values.
    ///
    /// If you want to check that a header name has multiple values, but do not mind if there are
    /// additional values, you can use `with_header` multiple times instead.
    ///
    /// If you want to ensure that a header name only has one value, you can use `with_header_once`
    /// instead.
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
    ///     .with_header_all("content-type", ["application/json", "text/html"])
    ///     .returning("OK")?;
    /// # Ok::<_, Error>(())
    /// # };
    /// ```
    ///
    /// ## Remark
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    pub fn with_header_all<K, IV, V>(self, key: K, values: IV) -> CaseBuilder<'c, WithHandler>
    where
        K: TryInto<HeaderName>,
        K::Error: Into<hyper::http::Error>,
        IV: IntoIterator<Item = V>,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        CaseBuilder {
            connector: self.connector,
            with: WithHandler::default().with_header_all(key, values),
            count: self.count,
        }
    }

    /// Match requests that contains the provided payload
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
    ///     .with_body("some body")
    ///     .returning("OK")?;
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
    pub fn with_body<B>(self, body: B) -> CaseBuilder<'c, WithHandler>
    where
        B: ToString,
    {
        CaseBuilder {
            connector: self.connector,
            with: Ok(WithHandler::default().with_body(body)),
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
    ///     .with_json(serde_json::json!({"status": "OK"}))
    ///     .returning("OK")?;
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
    pub fn with_json<V>(self, value: V) -> CaseBuilder<'c, WithHandler>
    where
        V: serde::Serialize,
    {
        CaseBuilder {
            connector: self.connector,
            with: WithHandler::default().with_json(value),
            count: self.count,
        }
    }

    /// Match requests that contains the provided JSON payload, but may contain other properties
    ///
    /// You can combine this with other validators, such as `with_uri`, but not with `with`.
    pub fn with_json_partial<V>(self, value: V) -> CaseBuilder<'c, WithHandler>
    where
        V: serde::Serialize,
    {
        CaseBuilder {
            connector: self.connector,
            with: WithHandler::default().with_json_partial(value),
            count: self.count,
        }
    }
}

impl<'c> CaseBuilder<'c, WithHandler> {
    #[doc(hidden)]
    pub fn with_uri<U>(mut self, uri: U) -> Self
    where
        U: TryInto<Uri>,
        U::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.and_then(|w| w.with_uri(uri));
        self
    }

    #[doc(hidden)]
    pub fn with_method<M>(mut self, method: M) -> Self
    where
        M: TryInto<Method>,
        M::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.and_then(|w| w.with_method(method));
        self
    }

    #[doc(hidden)]
    pub fn with_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<hyper::http::Error>,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.and_then(|w| w.with_header(key, value));
        self
    }

    #[doc(hidden)]
    pub fn with_header_once<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<hyper::http::Error>,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.and_then(|w| w.with_header_once(key, value));
        self
    }

    #[doc(hidden)]
    pub fn with_header_all<K, IV, V>(mut self, key: K, values: IV) -> Self
    where
        K: TryInto<HeaderName>,
        K::Error: Into<hyper::http::Error>,
        IV: IntoIterator<Item = V>,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.and_then(|w| w.with_header_all(key, values));
        self
    }

    #[doc(hidden)]
    pub fn with_body<B>(mut self, body: B) -> Self
    where
        B: ToString,
    {
        self.with = self.with.map(|w| w.with_body(body));
        self
    }

    #[doc(hidden)]
    #[cfg(feature = "json")]
    pub fn with_json<V>(mut self, value: V) -> Self
    where
        V: serde::Serialize,
    {
        self.with = self.with.and_then(|w| w.with_json(value));
        self
    }

    #[doc(hidden)]
    #[cfg(feature = "json")]
    pub fn with_json_partial<V>(mut self, value: V) -> Self
    where
        V: serde::Serialize,
    {
        self.with = self.with.and_then(|w| w.with_json_partial(value));
        self
    }
}

impl<'c, W> CaseBuilder<'c, W> {
    /// Mark how many times this mock case can be called
    ///
    /// Nothing enforces how many times a mock case is called, but you can use the `checkpoint`
    /// method on the [`Connector`] to ensure all methods were called the right amount of times.
    pub fn times(self, count: usize) -> Self {
        Self {
            count: Some(count),
            ..self
        }
    }
}

impl<'c, W> CaseBuilder<'c, W>
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
    ///
    /// ## Errors
    ///
    /// This will fail if any of the previous steps in [`CaseBuilder`] failed, or if it fails to
    /// store the case into the connector.
    pub fn returning<R>(self, returning: R) -> Result<(), Error>
    where
        R: Returning + 'static,
    {
        let case = Case::new(self.with?, returning, self.count);
        self.connector.cases.push(case);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use crate::Connector;

    use super::*;

    #[test]
    fn test_with() {
        let mut connector = Connector::builder();
        connector
            .expect()
            .with(|req: &Request<String>| Ok::<_, Infallible>(req.body().contains("hello")))
            .returning("OK")
            .unwrap();
    }
}
