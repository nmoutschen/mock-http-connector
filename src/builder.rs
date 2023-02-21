use crate::{
    case::Case,
    handler::{DefaultErrorHandler, DefaultWith, Returning, With},
    Connector,
};

pub struct Builder<F = DefaultErrorHandler> {
    cases: Vec<Case>,
    error_handler: F,
}

impl<F> Builder<F> {
    pub fn expect(&mut self) -> CaseBuilder<'_, F> {
        CaseBuilder::new(self)
    }

    pub fn error<NF>(self, error_handler: NF) -> Builder<NF> {
        Builder {
            cases: self.cases,
            error_handler,
        }
    }

    pub fn build(self) -> Connector<F> {
        Connector::new(self.cases, self.error_handler)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            cases: Default::default(),
            error_handler: DefaultErrorHandler,
        }
    }
}

pub struct CaseBuilder<'b, F, W = DefaultWith> {
    builder: &'b mut Builder<F>,
    with: W,
    count: Option<usize>,
}

impl<'b, F> CaseBuilder<'b, F> {
    fn new(builder: &'b mut Builder<F>) -> Self {
        Self {
            builder,
            with: DefaultWith,
            count: None,
        }
    }
}
impl<'b, F, W> CaseBuilder<'b, F, W> {
    pub fn with<NW>(self, with: NW) -> CaseBuilder<'b, F, NW> {
        CaseBuilder {
            builder: self.builder,
            with,
            count: self.count,
        }
    }

    pub fn times(self, count: usize) -> Self {
        Self {
            count: Some(count),
            ..self
        }
    }
}

impl<'b, F, W> CaseBuilder<'b, F, W>
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
