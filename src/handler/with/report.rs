use std::{borrow::Cow, collections::HashSet};

use hyper::http::HeaderName;

/// Report if a `with` clause for a case matched with an incoming request
///
/// This is used to generate debugging information when no cases match a request.
#[derive(Debug, Clone)]
pub enum Report {
    /// The request is a match
    Match,
    /// The request is a mismatch
    ///
    /// This can contain a [`HashSet`] of [`Reason`]s to help troubleshoot why the specific case
    /// didn't match the request.
    Mismatch(HashSet<Reason>),
}

impl From<bool> for Report {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Match,
            false => Self::Mismatch(HashSet::default()),
        }
    }
}

impl From<Vec<Reason>> for Report {
    fn from(value: Vec<Reason>) -> Self {
        if value.is_empty() {
            Self::Match
        } else {
            Self::Mismatch(value.into_iter().collect())
        }
    }
}

impl From<Option<Reason>> for Report {
    fn from(value: Option<Reason>) -> Self {
        match value {
            None => Self::Match,
            Some(reason) => {
                let mut reasons = HashSet::new();
                reasons.insert(reason);
                Self::Mismatch(reasons)
            }
        }
    }
}

/// Reason for mismatch on a case
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reason {
    /// Mismatch on the request method
    Method,
    /// Mismatch on the request URI
    Uri,
    /// Mismatch on one header
    Header(HeaderName),
    /// Mismatch on the payload body
    Body,
}

impl Reason {
    /// Returns a string representation for the [`Reason`]
    pub fn as_str(&self) -> Cow<'static, str> {
        match self {
            Self::Method => "method".into(),
            Self::Uri => "uri".into(),
            Self::Header(name) => format!("header `{name}`").into(),
            Self::Body => "body".into(),
        }
    }
}
