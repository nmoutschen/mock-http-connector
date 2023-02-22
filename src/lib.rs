#![warn(missing_docs)]

//! # Mock connector for [`hyper::Client`]
//!
//! This crate provides a mock [`Connector`] to replace the default one when testing applications
//! that makes HTTP calls using [`hyper`].
//!
//! ## Usage
//!
//! ```rust
//! # use hyper::{Body, Request};
//! # use mock_http_connector::{Connector, Error};
//! # tokio_test::block_on(async move {
//! // Create a mock Connector
//! let mut builder = Connector::builder();
//! builder
//!     .expect()
//!     .times(1)
//!     .with_uri("https://example.com/test")?
//!     .returning("OK");
//! let connector = builder.build();
//!
//! // Use it when creating the hyper Client
//! let client = hyper::Client::builder().build::<_, Body>(connector.clone());
//!
//! // Send requests as normal
//! let _res = client
//! .request(
//!     Request::builder()
//!         .uri("http://example.com/test")
//!         .body("".to_string().into())?,
//! )
//! .await
//! .unwrap();
//!
//! // Check if all expectations were called the right number of times
//! connector.checkpoint()?;
//!
//! # Ok::<_, Error>(())
//! # });
//! ```

mod builder;
mod case;
mod connector;
mod error;
mod handler;
mod stream;

pub use builder::{Builder, CaseBuilder};
use case::Case;
pub use connector::Connector;
pub use error::Error;
pub use handler::Returning;
