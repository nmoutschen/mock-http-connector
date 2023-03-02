# Mock connector for [`hyper::Client`]

This crate provides a mock [`Connector`] to replace the default one when testing applications
that makes HTTP calls using [`hyper`].

## Usage

```rust
# use hyper::{Body, Request};
# use mock_http_connector::{Connector, Error};
# tokio_test::block_on(async move {
// Create a mock Connector
let connector = Connector::new();
connector
    .expect()
    .times(1)
    .with_uri("https://example.com/test")
    .returning("OK")?;

// Use it when creating the hyper Client
let client = hyper::Client::builder().build::<_, Body>(connector.clone());

// Send requests as normal
let _res = client
.request(
    Request::builder()
        .uri("https://example.com/test")
        .body("".to_string().into())?,
)
.await?;

// Check if all expectations were called the right number of times
connector.checkpoint()?;

# Ok::<_, Error>(())
# });
```