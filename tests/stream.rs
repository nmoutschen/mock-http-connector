use mock_http_connector::Connector;
use rstest::*;
use speculoos::prelude::*;
use std::{
    convert::Infallible,
    error::Error as StdError,
    task::{Context, Poll},
};
mod helpers;
use helpers::*;

#[rstest]
#[tokio::test]
async fn test_body_stream() -> Result<(), Box<dyn StdError>> {
    // GIVEN
    // * a connector that expects a chunk-encoded payload
    // * a Body wrapping a stream that splits the payload into 2 str
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_method("POST")
        .with_header("transfer-encoding", "chunked")
        .with_body("6\r\nhello \r\n6\r\nworld!\r\n0\r\n\r\n")
        .returning((202, "OK"))?;

    let connector = builder.build();

    #[cfg(feature = "hyper_0_14")]
    let client = client(connector.clone());
    #[cfg(feature = "hyper_1")]
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(connector.clone());
    #[cfg(feature = "hyper_0_14")]
    let stream = tokio_stream::iter(vec![Ok::<_, Infallible>("hello "), Ok("world!")]);
    #[cfg(feature = "hyper_1")]
    let stream = tokio_stream::iter(vec![
        Ok::<_, Infallible>(hyper_1::body::Frame::data("hello ".as_bytes())),
        Ok(hyper_1::body::Frame::data("world!".as_bytes())),
    ]);

    // WHEN making a request that sends a stream of data
    #[cfg(feature = "hyper_0_14")]
    let res = client
        .request(
            Request::builder()
                .method("POST")
                .uri("http://test.example")
                .body(hyper_0_14::Body::wrap_stream(stream))?,
        )
        .await;
    #[cfg(feature = "hyper_1")]
    let res = client
        .request(
            Request::builder()
                .method("POST")
                .uri("http://test.example")
                .body(http_body_util::StreamBody::new(stream))?,
        )
        .await;

    // THEN it returns the right payload
    assert_that!(res).is_ok().matches(|res| res.status() == 202);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_stream() -> Result<(), Box<dyn StdError>> {
    // GIVEN
    // * a connector that expects a chunk-encoded payload
    // * a custom HttpBody implementation that returns 2 payloads
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_method("POST")
        .with_header("transfer-encoding", "chunked")
        .with_body("6\r\nworld!\r\n6\r\nhello \r\n0\r\n\r\n")
        .returning((202, "OK"))?;

    let connector = builder.build();

    #[cfg(feature = "hyper_0_14")]
    let client = hyper_0_14::Client::builder().build(connector.clone());
    #[cfg(feature = "hyper_1")]
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(connector.clone());
    let body = CustomBody::new(["hello ", "world!"]);

    // WHEN making a request that sends a stream of data
    let res = client
        .request(
            Request::builder()
                .method("POST")
                .uri("http://test.example")
                .body(body)?,
        )
        .await;

    // THEN it returns the right payload
    assert_that!(res).is_ok().matches(|res| res.status() == 202);

    Ok(())
}

struct CustomBody {
    data: Vec<&'static str>,
}

impl CustomBody {
    pub fn new<D>(data: D) -> Self
    where
        D: IntoIterator<Item = &'static str>,
    {
        Self {
            data: data.into_iter().collect(),
        }
    }
}

#[cfg(feature = "hyper_0_14")]
impl hyper_0_14::body::HttpBody for CustomBody {
    type Data = hyper_0_14::body::Bytes;
    type Error = Infallible;

    fn poll_data(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.data.pop() {
            Some(data) => Poll::Ready(Some(Ok(hyper_0_14::body::Bytes::from(data)))),
            None => Poll::Ready(None),
        }
    }

    fn poll_trailers(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<hyper_0_14::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

#[cfg(feature = "hyper_1")]
impl hyper_1::body::Body for CustomBody {
    type Data = hyper_1::body::Bytes;
    type Error = Infallible;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper_1::body::Frame<Self::Data>, Self::Error>>> {
        match self.data.pop() {
            Some(data) => Poll::Ready(Some(Ok(hyper_1::body::Frame::data(data.into())))),
            None => Poll::Ready(None),
        }
    }
}
