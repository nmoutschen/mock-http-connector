use hyper::{
    http::{HeaderName, HeaderValue},
    Method, Uri,
};

/// Reason why a call to [`crate::With`] returned a mismatch
#[derive(Clone, Debug)]
pub enum Reason<'w> {
    /// Mismatch on the request body
    Body {
        /// Start of the mismatch
        start: Option<usize>,
        /// Length of the mismatch
        length: Option<usize>,
        /// Optional message explaining the mismatch
        message: Option<String>,
    },
    /// Mismatch on the request header(s)
    Header {
        /// Header name
        name: &'w HeaderName,
        /// Expected header value
        expected: &'w HeaderValue,
        /// Optional message explaining the mismatch
        message: Option<String>,
    },
    /// Mismatch on the request method
    Method {
        /// Expected [`Method`]
        expected: &'w Method,
    },
    /// Mismatch on the URI
    Uri {
        /// Expected URI
        expected: &'w Uri,
        /// Optional message explaining the mismatch
        message: Option<String>,
    },
}

impl<'w> Reason<'w> {
    /// Create a new [`Reason`] for a body mismatch
    pub fn body(message: Option<String>) -> Self {
        Self::Body {
            start: None,
            length: None,
            message,
        }
    }

    /// Create a new [`Reason`] for a body mismatch with a position
    pub fn body_with_pos(start: usize, length: usize, message: Option<String>) -> Self {
        Self::Body {
            start: Some(start),
            length: Some(length),
            message,
        }
    }

    /// Create a new [`Reason`] for a header mismatch
    pub fn header(
        name: &'w HeaderName,
        expected: &'w HeaderValue,
        message: Option<String>,
    ) -> Self {
        Self::Header {
            name,
            expected,
            message,
        }
    }

    /// Create a new [`Reason`] for a method mismatch
    pub fn method(expected: &'w Method) -> Self {
        Self::Method { expected }
    }

    /// Create a new [`Reason`] for an URI mismatch
    pub fn uri(expected: &'w Uri, message: Option<String>) -> Self {
        Self::Uri { expected, message }
    }
}
