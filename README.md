# Mock connector for [`hyper::Client`]

This crate provides a mock [`Connector`] to replace the default one when testing applications
that makes HTTP calls using [`hyper`].

## Usage

```rust
# #[cfg(feature = "hyper_0_14")]
# use hyper_0_14::{Body, Request};
# #[cfg(feature = "hyper_1")]
# use hyper_1::{body::Bytes, Request};
# #[cfg(feature = "hyper_1")]
# use http_body_util::Full;
# #[cfg(feature = "hyper_1")]
# use hyper_util::rt::TokioExecutor;
# use mock_http_connector::{Connector, Error};
# tokio_test::block_on(async move {
// Create a mock Connector
let mut builder = Connector::builder();
builder
    .expect()
    .times(1)
    .with_uri("https://example.com/test")
    .returning("OK")?;
let connector = builder.build();

// Use it when creating the hyper Client
#[cfg(feature = "hyper_0_14")]
let client = hyper_0_14::Client::builder().build::<_, Body>(connector.clone());
#[cfg(feature = "hyper_1")]
let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(connector.clone());

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

## Reporting

In case a Request does not match any of the cases defined in the mock connector, this crate can
print a report showing why each case didn't match.

For example:

```text
--> no matching case for request
 | 
 = the incoming request did not match any know cases.
 = incoming request:
 | 
 | method:   GET
 | uri:      http://test.example/
 | headers:
 |   authorization: bearer 1234
 |   host         : test.example
 | 
--> case 0 `WithHandler`
 | 
 | method:   POST
 |           ^^^^
 | uri:      http://test.example/
 | headers:
 |   authorization: bearer 1234
 |   content-type : application/json
 |                  ^^^^^^^^^^^^^^^^
 | body:
 | > some multi-line payload
 | > on 2 lines
 |   ^^^^^^^^^^^^^^^^^^^^^^^
 | 
 = this case doesn't match the request on the following attributes:
 | - method
 | - body
 | - header `content-type`
 | 
--> case 1 `WithHandler`
 | 
 | method:   GET
 | uri:      http://test.example/some-path
 |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 | headers:
 |   authorization: bearer 1234
 | 
 = this case doesn't match the request on the following attributes:
 | - uri
 | 
```