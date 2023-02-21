use std::{
    future::{ready, Ready},
    io,
    str::from_utf8,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use hyper::{service::Service, Request, Response, Uri};

use crate::{
    handler::{DefaultErrorHandler, ErrorHandler},
    stream::MockStream,
    Builder, Case, Error,
};

#[derive(Default)]
pub struct Connector<F = DefaultErrorHandler> {
    cases: Arc<Mutex<Vec<Case>>>,
    error_handler: Arc<F>,
}

impl<F> Clone for Connector<F> {
    fn clone(&self) -> Self {
        Self {
            cases: self.cases.clone(),
            error_handler: self.error_handler.clone(),
        }
    }
}

impl Connector {
    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl<F> Connector<F> {
    pub(crate) fn new(cases: Vec<Case>, error_handler: F) -> Self {
        Self {
            cases: Arc::new(Mutex::new(cases)),
            error_handler: Arc::new(error_handler),
        }
    }

    pub(crate) fn matches(
        &self,
        req: httparse::Request,
        body: &[u8],
    ) -> Result<Option<Response<String>>, Error> {
        let mut cases = self.cases.lock()?;

        let req = into_request(req, body);

        for case in cases.iter_mut() {
            let res = case.with.with(&req)?;
            if res {
                case.seen += 1;
                return Ok(Some(case.returning.returning(req)?));
            }
        }

        Ok(None)
    }

    pub fn checkpoint(&self) -> Result<(), Error> {
        let cases = self.cases.lock()?;
        let checkpoints = cases
            .iter()
            .filter_map(|case| case.checkpoint())
            .collect::<Vec<_>>();

        if checkpoints.is_empty() {
            Ok(())
        } else {
            Err(Error::Checkpoint(checkpoints))
        }
    }
}

impl<F> Connector<F>
where
    F: ErrorHandler,
{
    pub(crate) fn error(&self) -> Response<String> {
        self.error_handler.handle_error()
    }
}

impl<F> Service<Uri> for Connector<F> {
    type Response = MockStream<F>;
    type Error = io::Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Uri) -> Self::Future {
        ready(Ok(MockStream::new(self.clone())))
    }
}

fn into_request(req: httparse::Request, body: &[u8]) -> Request<String> {
    let body = from_utf8(body).unwrap().to_string();

    let mut builder = Request::builder();

    if let Some(method) = req.method {
        builder = builder.method(method);
    }
    for header in req.headers {
        if !header.name.is_empty() {
            dbg!(&header);
            builder = builder.header(header.name, header.value);
        }
    }

    builder.body(body).unwrap()
}
