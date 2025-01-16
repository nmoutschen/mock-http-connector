use mock_http_connector::Connector;
use rstest::*;
use speculoos::prelude::*;
use std::{error::Error as StdError, str::from_utf8};
mod helpers;
use helpers::*;

#[rstest]
#[tokio::test]
async fn test_async() -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a connector with an async function
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_uri("http://test.example")
        .returning(|_req| async { "hello" })?;
    let connector = builder.build();
    let client = client(connector.clone());

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

    let body = to_bytes(res?.body_mut()).await;
    let body = from_utf8(&body)?;

    assert_that!(body).is_equal_to("hello");
    connector.checkpoint()?;

    Ok(())
}

#[cfg(feature = "json")]
#[rstest]
#[tokio::test]
async fn test_json() -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a connector returning a json payload
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_uri("http://test.example")
        .returning(serde_json::json!({"value": 3}))?;
    let connector = builder.build();

    let client = client(connector.clone());

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

    let body = to_bytes(res?.body_mut()).await;
    let body: serde_json::Value = serde_json::from_slice(&body)?;

    assert_that!(body).is_equal_to(serde_json::json!({"value": 3}));
    connector.checkpoint()?;

    Ok(())
}
