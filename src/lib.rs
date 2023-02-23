#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

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
