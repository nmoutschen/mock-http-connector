use hyper::{body::to_bytes, Body, Request};
use mock_http_connector::Connector;
use rstest::*;
use speculoos::prelude::*;
use std::{error::Error as StdError, str::from_utf8};

#[rstest]
#[tokio::test]
async fn test_async() -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a connector with an async function
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_uri("http://test.example")?
        .returning(|_req| async { "hello" });
    let connector = builder.build();
    let client = hyper::Client::builder().build::<_, Body>(connector.clone());

    // WHEN making a request
    let res = client
        .request(
            Request::builder()
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns the right payload
    assert_that!(res).is_ok();

    let body = to_bytes(res?.body_mut()).await?;
    let body = from_utf8(&body)?;

    assert_that!(body).is_equal_to("hello");
    connector.checkpoint()?;

    Ok(())
}
