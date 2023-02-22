use crate::{
    case::Case,
    handler::{
        DefaultErrorHandler, DefaultMissingHandler, DefaultWith, Returning, With, WithHandler,
    },
    Connector, Error,
};
use hyper::{header::IntoHeaderName, http::HeaderValue, Method, Uri};

pub struct Builder<FE = DefaultErrorHandler, FM = DefaultMissingHandler> {
    cases: Vec<Case>,
    error_handler: FE,
    missing_handler: FM,
}

impl<FE, FM> Builder<FE, FM> {
    pub fn expect(&mut self) -> CaseBuilder<'_, FE, FM> {
        CaseBuilder::new(self)
    }

    pub fn error<NF>(self, error_handler: NF) -> Builder<NF, FM> {
        Builder {
            cases: self.cases,
            error_handler,
            missing_handler: self.missing_handler,
        }
    }

    pub fn missing<NF>(self, missing_handler: NF) -> Builder<FE, NF> {
        Builder {
            cases: self.cases,
            error_handler: self.error_handler,
            missing_handler,
        }
    }

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

    pub fn with<W>(self, with: W) -> CaseBuilder<'b, FE, FM, W> {
        CaseBuilder {
            builder: self.builder,
            with,
            count: self.count,
        }
    }

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

    #[cfg(feature = "json")]
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
}

impl<'b, FE, FM> CaseBuilder<'b, FE, FM, WithHandler> {
    pub fn with_uri<U>(mut self, uri: U) -> Result<Self, Error>
    where
        U: TryInto<Uri>,
        U::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.with_uri(uri)?;
        Ok(self)
    }

    pub fn with_method<M>(mut self, method: M) -> Result<Self, Error>
    where
        M: TryInto<Method>,
        M::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.with_method(method)?;
        Ok(self)
    }

    pub fn with_header<K, V>(mut self, key: K, value: V) -> Result<Self, Error>
    where
        K: IntoHeaderName,
        V: TryInto<HeaderValue>,
        V::Error: Into<hyper::http::Error>,
    {
        self.with = self.with.with_header(key, value)?;
        Ok(self)
    }

    pub fn with_body<B>(mut self, body: B) -> Self
    where
        B: ToString,
    {
        self.with = self.with.with_body(body);
        self
    }

    #[cfg(feature = "json")]
    pub fn with_json<V>(mut self, value: V) -> Result<Self, Error>
    where
        V: serde::Serialize,
    {
        self.with = self.with.with_json(value)?;
        Ok(self)
    }
}

impl<'b, FE, FM, W> CaseBuilder<'b, FE, FM, W> {
    pub fn times(self, count: usize) -> Self {
        Self {
            count: Some(count),
            ..self
        }
    }
}

impl<'b, FE, FM, W> CaseBuilder<'b, FE, FM, W>
where
    W: With + Send + Sync + 'static,
{
    pub fn returning<R>(self, returning: R)
    where
        R: Returning + Send + Sync + 'static,
    {
        let case = Case::new(self.with, returning, self.count);
        self.builder.cases.push(case);
    }
}
