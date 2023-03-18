use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::handler::{Returning, With};

#[derive(Clone)]
pub(crate) struct Case {
    pub(crate) with: Arc<Box<dyn With + Send + Sync>>,
    pub(crate) returning: Arc<Box<dyn Returning + Send + Sync>>,
    count: Option<usize>,
    pub(crate) seen: Arc<AtomicUsize>,
}

impl Case {
    pub fn new<W, R>(with: W, returning: R, count: Option<usize>) -> Self
    where
        W: With + Send + Sync + 'static,
        R: Returning + Send + Sync + 'static,
    {
        Self {
            with: Arc::new(Box::new(with)),
            returning: Arc::new(Box::new(returning)),
            count,
            seen: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn checkpoint(&self) -> Option<Checkpoint> {
        self.count
            .and_then(|count| Checkpoint::check(count, self.seen.load(Ordering::Acquire)))
    }
}

#[derive(Debug)]
pub struct Checkpoint {
    expected: usize,
    got: usize,
}

impl std::fmt::Display for Checkpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected {}, got {}", self.expected, self.got)
    }
}

impl Checkpoint {
    pub fn check(expected: usize, got: usize) -> Option<Self> {
        if expected == got {
            None
        } else {
            Some(Self { expected, got })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use hyper::{Request, Response, StatusCode};

    use super::*;

    #[test]
    fn case_new() {
        let _case = Case::new(
            |_req: &Request<String>| Ok::<_, Infallible>(true),
            |_| async { Response::builder().status(StatusCode::OK).body("") },
            None,
        );
    }
}
