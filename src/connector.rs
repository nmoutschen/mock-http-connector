use colored::Colorize;
use hyper::{service::Service, Request, Uri};
use std::{
    cmp::max,
    collections::{BinaryHeap, HashSet},
    future::{ready, Ready},
    io,
    str::from_utf8,
    sync::{atomic::Ordering, Arc},
    task::{Context, Poll},
};

use crate::{
    builder::Builder, error::BoxError, response::ResponseFuture, stream::MockStream, Case, Error,
    Level, Reason, Report,
};

/// Mock connector for [`hyper::Client`]
///
/// See the crate documentation for how to configure the connector.
#[derive(Default, Clone)]
pub struct Connector {
    inner: Arc<InnerConnector>,
}

impl Connector {
    /// Create a new [`Builder`]
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Check if all the mock cases were called the right amount of time
    ///
    /// If not, this will return an error with all the mock cases that failed.
    pub fn checkpoint(&self) -> Result<(), Error> {
        self.inner.checkpoint()
    }

    pub(crate) fn from_inner(inner: InnerConnector) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

#[derive(Default)]
pub(crate) struct InnerConnector {
    pub level: Level,
    pub cases: Vec<Case>,
}

impl InnerConnector {
    pub fn checkpoint(&self) -> Result<(), Error> {
        let checkpoints = self
            .cases
            .iter()
            .filter_map(|case| case.checkpoint())
            .collect::<Vec<_>>();

        if checkpoints.is_empty() {
            Ok(())
        } else {
            Err(Error::Checkpoint(checkpoints))
        }
    }

    pub(crate) fn matches(
        &self,
        req: httparse::Request,
        body: &[u8],
        uri: &Uri,
    ) -> Result<ResponseFuture, Error> {
        let req = into_request(req, body, uri)?;

        let mut reports = Vec::new();

        for case in self.cases.iter() {
            match case.with.with(&req)? {
                Report::Match => {
                    case.seen.fetch_add(1, Ordering::Release);
                    return Ok(case.returning.returning(req));
                }
                Report::Mismatch(reasons) => {
                    reports.push((case, reasons));
                }
            }
        }

        // Couldn't find a match, log the error
        if self.level >= Level::Missing {
            print_report(&req, reports);
        }
        Err(Error::NotFound(req))
    }
}

impl Service<Uri> for Connector {
    type Response = MockStream;
    type Error = io::Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        ready(Ok(MockStream::new(self.inner.clone(), req)))
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

fn print_report(req: &Request<String>, reports: Vec<(&Case, HashSet<Reason>)>) {
    let req_note = " = ".red().bold();
    let req_bar = " | ".red().bold();
    let case_note = " = ".blue().bold();
    let case_bar = " | ".blue().bold();

    println!("{}", "--> no matching case for request".red().bold());
    println!("{req_bar}");
    println!("{req_note}the incoming request did not match any know cases.");
    println!("{req_note}incoming request:");
    println!("{req_bar}");
    println!("{req_bar}method:   {}", req.method());
    println!("{req_bar}uri:      {}", req.uri());
    if !req.headers().is_empty() {
        let key_length = req
            .headers()
            .iter()
            .fold(0, |acc, (key, _)| max(acc, key.to_string().len()));
        println!("{req_bar}headers:");
        for (key, value) in req.headers() {
            let value = if let Ok(value) = value.to_str() {
                value.into()
            } else {
                format!("{value:?}")
            };
            println!("{req_bar}  {key: <key_length$}: {value}");
        }
    }
    println!("{req_bar}");

    if !req.body().is_empty() {
        println!("{req_bar}{}:", "body".bold());
        for line in req.body().split('\n') {
            println!("{req_bar}{line}");
        }
        println!("{req_bar}");
    }

    for (id, (case, report)) in reports.iter().enumerate() {
        let with_print = case.with.print_pretty(report);
        println!(
            "{}",
            format!("--> case {id} `{}`", with_print.name).blue().bold(),
        );
        if let Some(body) = with_print.body {
            println!("{case_bar}");
            for line in body.split('\n') {
                println!("{case_bar}{line}");
            }
            println!("{case_bar}");
        }
        if !report.is_empty() {
            let cases = report.iter().map(|r| r.as_str()).collect::<BinaryHeap<_>>();
            println!("{case_note}this case doesn't match the request on the following attributes:");
            for case in cases {
                println!("{case_bar}- {case}");
            }
            println!("{case_bar}");
        }
    }

    println!();
}
