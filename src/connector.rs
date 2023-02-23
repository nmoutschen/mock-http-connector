use std::{
    future::{ready, Ready},
    io,
    str::from_utf8,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use hyper::{service::Service, Request, Response, Uri};

use crate::{
    error::BoxError,
    handler::{DefaultErrorHandler, DefaultHandler, DefaultMissingHandler},
    stream::MockStream,
    Builder, Case, Error,
};

/// Mock connector for [`hyper::Client`]
///
/// See the crate documentation for how to configure the connector.
#[derive(Default)]
pub struct Connector<FE = DefaultErrorHandler, FM = DefaultMissingHandler> {
    cases: Arc<Mutex<Vec<Case>>>,
    error_handler: Arc<FE>,
    missing_handler: Arc<FM>,
}

impl<FE, FM> Clone for Connector<FE, FM> {
    fn clone(&self) -> Self {
        Self {
            cases: self.cases.clone(),
            error_handler: self.error_handler.clone(),
            missing_handler: self.missing_handler.clone(),
        }
    }
}

impl Connector {
    /// Create a new [`Builder`] to specify expected [`Request`]s and their corresponding
    /// [`Response`]s
    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl<FE, FM> Connector<FE, FM> {
    pub(crate) fn new(cases: Vec<Case>, error_handler: FE, missing_handler: FM) -> Self {
        Self {
            cases: Arc::new(Mutex::new(cases)),
            error_handler: Arc::new(error_handler),
            missing_handler: Arc::new(missing_handler),
        }
    }

    pub(crate) fn matches(
        &self,
        req: httparse::Request,
        body: &[u8],
        uri: &Uri,
    ) -> Result<Option<Response<String>>, Error> {
        let mut cases = self.cases.lock()?;

        let req = into_request(req, body, uri)?;

        for case in cases.iter_mut() {
            let res = case.with.with(&req)?;
            if res {
                case.seen += 1;
                return Ok(Some(case.returning.returning(req)?));
            }
        }

        // Couldn't find a match, log the error
        eprintln!("no match found for request {req:?}");
        Ok(None)
    }

    /// Check if all the mock cases were called the right amount of time
    ///
    /// If not, this will return an error with all the mock cases that failed.
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

impl<FE, FM> Connector<FE, FM>
where
    FE: DefaultHandler,
{
    pub(crate) fn error(&self) -> Response<String> {
        self.error_handler.handle()
    }
}

impl<FE, FM> Connector<FE, FM>
where
    FM: DefaultHandler,
{
    pub(crate) fn missing(&self) -> Response<String> {
        self.missing_handler.handle()
    }
}

impl<FE, FM> Service<Uri> for Connector<FE, FM> {
    type Response = MockStream<FE, FM>;
    type Error = io::Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        ready(Ok(MockStream::new(self.clone(), req)))
    }
}

fn into_request(
    req: httparse::Request,
    body: &[u8],
    uri: &Uri,
) -> Result<Request<String>, BoxError> {
    let body = from_utf8(body)?.to_string();

    let mut builder = Request::builder().uri(uri);

    if let Some(path) = req.path {
        // TODO: handle errors
        let mut parts = uri.clone().into_parts();
        parts.path_and_query = Some(path.parse()?);
        builder = builder.uri(Uri::from_parts(parts)?);
    }
    if let Some(method) = req.method {
        builder = builder.method(method);
    }
    for header in req.headers {
        if !header.name.is_empty() {
            builder = builder.header(header.name, header.value);
        }
    }

    Ok(builder.body(body)?)
}
