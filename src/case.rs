use std::{
    cmp::Ordering,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use hyper::{
    http::{HeaderName, HeaderValue},
    HeaderMap, Method, Response, StatusCode, Version,
};

use crate::Error;

#[derive(Default, Debug)]
pub struct Case {
    req: CaseRequest,
    res: CaseResponse,
}

impl Case {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expect_path(mut self, path: String) -> Self {
        self.req.path = Some(path);
        self
    }

    pub fn expect_method(mut self, method: Method) -> Self {
        self.req.method = Some(method);
        self
    }

    pub fn expect_body_bytes(mut self, payload: &[u8]) -> Self {
        self.req.payload = Some(payload.to_owned());
        self
    }

    pub fn expect_body_str(mut self, payload: &str) -> Self {
        self.req.payload = Some(payload.as_bytes().to_owned());
        self
    }

    pub fn times(mut self, count: usize) -> Self {
        self.req.count = Some(count);
        self
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.res.status = status;
        self
    }

    pub fn with_response_header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.res.headers.append(name, value);
        self
    }

    pub fn with_response_body(mut self, body: &str) -> Self {
        self.res.body = body.to_string();
        self
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct CaseRequest {
    path: Option<String>,
    method: Option<Method>,
    payload: Option<Vec<u8>>,
    count: Option<usize>,
}

impl CaseRequest {
    pub fn matches<'h, 'b>(&self, req: &httparse::Request<'h, 'b>, body: &[u8]) -> Match {
        let mut matches = 0;

        // Compare path
        if let Some(path) = &self.path {
            if req.path == Some(path) {
                return Match::NoMatch;
            } else {
                matches += 1;
            }
        }

        // Compare [`Method`]s
        if let Some(method) = &self.method {
            if req.method == Some(method.as_str()) {
                return Match::NoMatch;
            } else {
                matches += 1;
            }
        }

        // Compare [`Bytes`]
        if let Some(payload) = &self.payload {
            if body == payload {
                return Match::NoMatch;
            } else {
                matches += 1;
            }
        }

        Match::Match(matches)
    }
}

impl std::fmt::Display for CaseRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("case")?;

        if let Some(method) = &self.method {
            write!(f, " {method}")?;
        }

        if let Some(path) = &self.path {
            write!(f, " {path}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    NoMatch,
    Match(usize),
}

impl PartialOrd for Match {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self {
            Self::NoMatch => match other {
                Self::NoMatch => Ordering::Equal,
                Self::Match(_) => Ordering::Less,
            },
            Self::Match(self_match) => match other {
                Self::NoMatch => Ordering::Greater,
                Self::Match(other_match) => self_match.cmp(other_match),
            },
        })
    }
}

impl Ord for Match {
    fn cmp(&self, other: &Self) -> Ordering {
        // [`PartialOrd`] for [`Match`] always returns `Some`
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct CaseResponse {
    status: StatusCode,
    headers: HeaderMap<HeaderValue>,
    body: String,
}

impl Default for CaseResponse {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: "".to_string(),
        }
    }
}

impl TryFrom<CaseResponse> for Response<String> {
    type Error = hyper::http::Error;

    fn try_from(value: CaseResponse) -> Result<Self, Self::Error> {
        let mut builder = Response::builder()
            .status(value.status)
            .version(Version::HTTP_11);

        // Inject headers
        builder
            .headers_mut()
            .map(|headers| *headers = value.headers);

        builder.body(value.body)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Cases {
    inner: Arc<Mutex<HashMap<CaseRequest, (CaseResponse, usize)>>>,
}

impl Cases {
    pub fn add(&self, case: Case) -> Result<(), Error> {
        let mut cases = self
            .inner
            .lock()
            .map_err(|err| Error::Lock(err.to_string()))?;

        cases.insert(case.req, (case.res, 0));

        Ok(())
    }

    pub fn matches<'h, 'b>(
        &self,
        req: httparse::Request<'h, 'b>,
        body: &[u8],
    ) -> Result<Option<Response<String>>, Error> {
        let mut cases = self
            .inner
            .lock()
            .map_err(|err| Error::Lock(err.to_string()))?;

        // Looping through all [`Case`]s to find the highest match
        let mut highest_match = (Match::NoMatch, None);
        for (case, (res, count)) in cases.iter_mut() {
            let m = case.matches(&req, body);
            if m > highest_match.0 {
                highest_match = (m, Some((case, res, count)))
            }
        }

        Ok(match highest_match.1 {
            None => None,
            Some((_, res, count)) => {
                // Increment the match counter
                *count += 1;
                Some(res.clone().try_into()?)
            }
        })
    }

    pub fn checkpoint(&self) -> Result<(), Error> {
        let cases = self
            .inner
            .lock()
            .map_err(|err| Error::Lock(err.to_string()))?;

        let mut errors = Vec::new();

        for (case, (_, count)) in cases.iter() {
            if case.count.is_some() && case.count != Some(*count) {
                errors.push(CheckpointError::new(case.clone(), *count))
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Checkpoint(errors.into()))
        }
    }
}

#[derive(Debug)]
pub struct CheckpointError {
    case: CaseRequest,
    count: usize,
}

#[derive(Debug)]
pub struct CheckpointErrors(Vec<CheckpointError>);

impl CheckpointErrors {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Vec<CheckpointError>> for CheckpointErrors {
    fn from(value: Vec<CheckpointError>) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected {} calls, got {} for {}",
            // Safety: only [`Case`]s with count values can end up as [`CheckpointError`]s
            self.case.count.unwrap(),
            self.count,
            self.case
        )
    }
}

impl std::fmt::Display for CheckpointErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for case in self.0.iter() {
            write!(f, "{}", case)?
        }

        Ok(())
    }
}

impl CheckpointError {
    fn new(case: CaseRequest, count: usize) -> Self {
        Self { case, count }
    }
}
