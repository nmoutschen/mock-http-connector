#![cfg(feature = "hyper_0_14")]

use hyper_0_14::{http::HeaderName, Body, Method, Request};
use mock_http_connector::Connector;
use rstest::*;
use speculoos::prelude::*;
use std::error::Error as StdError;

#[rstest]
#[case(Method::POST)] // Remark: hyper defaults to GET
#[case(Method::OPTIONS)]
#[tokio::test]
async fn test_method(#[case] method: Method) -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a connector expecting a Method
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_method(method.clone())
        .returning((202, "OK"))?;
    let connector = builder.build();

    let client = hyper_0_14::Client::builder().build::<_, Body>(connector.clone());

    // WHEN making a request with the right Method
    let res = client
        .request(
            Request::builder()
                .method(method)
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns the right payload
    assert_that!(res).is_ok().matches(|res| res.status() == 202);

    // WHEN making a request without the Method
    let res = client
        .request(
            Request::builder()
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns an error
    assert_that!(res).is_err();

    Ok(())
}

#[rstest]
#[case(hyper_0_14::header::ACCEPT, "application/json")]
#[case(hyper_0_14::header::AUTHORIZATION, "Bearer some-token")]
#[tokio::test]
async fn test_header(
    #[case] name: HeaderName,
    #[case] value: &'static str,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a connector expecting a header
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_header(name.clone(), value)
        .returning((202, "OK"))?;

    let connector = builder.build();

    let client = hyper_0_14::Client::builder().build::<_, Body>(connector.clone());

    // WHEN making a request with the right header
    let res = client
        .request(
            Request::builder()
                .header(name.clone(), value)
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns the right payload
    assert_that!(res).is_ok().matches(|res| res.status() == 202);

    // WHEN making a request without the header
    let res = client
        .request(
            Request::builder()
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns an error
    assert_that!(res).is_err();

    // WHEN making a request with a different value for the header
    let res = client
        .request(
            Request::builder()
                .header(name, "SOME OTHER VALUE")
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns an error
    assert_that!(res).is_err();

    Ok(())
}

#[rstest]
#[case(
    hyper_0_14::header::ACCEPT,
    "application/json",
    hyper_0_14::header::AUTHORIZATION,
    "Bearer some-token"
)]
#[tokio::test]
async fn test_headers(
    #[case] name_1: HeaderName,
    #[case] value_1: &'static str,
    #[case] name_2: HeaderName,
    #[case] value_2: &'static str,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a connector expecting two headers
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_header(name_1.clone(), value_1)
        .with_header(name_2.clone(), value_2)
        .returning((202, "OK"))?;
    let connector = builder.build();

    let client = hyper_0_14::Client::builder().build::<_, Body>(connector.clone());

    // WHEN making a request with the right headers
    let res = client
        .request(
            Request::builder()
                .header(name_1.clone(), value_1)
                .header(name_2, value_2)
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns the right payload
    assert_that!(res).is_ok().matches(|res| res.status() == 202);

    // WHEN making a request with just one header
    let res = client
        .request(
            Request::builder()
                .header(name_1, value_1)
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns an error
    assert_that!(res).is_err();

    Ok(())
}

#[rstest]
#[case("http://example.test/some/path")]
#[case("https://test.example/some/other/path")]
#[tokio::test]
async fn test_uri(#[case] uri: &'static str) -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a connector expecting an URI
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_uri(uri)
        .returning((202, "OK"))?;
    let connector = builder.build();

    let client = hyper_0_14::Client::builder().build::<_, Body>(connector.clone());

    // WHEN making a request with the right URI
    let res = client
        .request(Request::builder().uri(uri).body("".to_string().into())?)
        .await;

    // THEN it returns the right payload
    assert_that!(res).is_ok().matches(|res| res.status() == 202);

    // WHEN making a request with a different uri
    let res = client
        .request(
            Request::builder()
                .uri("http://invalid.uri")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns an error
    assert_that!(res).is_err();

    Ok(())
}
