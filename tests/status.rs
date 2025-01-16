use mock_http_connector::Connector;
use rstest::*;
use speculoos::prelude::*;
use std::error::Error as StdError;
mod helpers;
use helpers::*;

#[rstest]
#[case(StatusCode::ACCEPTED)]
#[case(StatusCode::SEE_OTHER)]
#[case(StatusCode::NOT_FOUND)]
#[case(StatusCode::NOT_ACCEPTABLE)]
#[case(StatusCode::INTERNAL_SERVER_ERROR)]
#[tokio::test]
async fn test_status_u16(
    #[case] status: StatusCode,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a client that returns `status` as an u16
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_uri("http://test.example")
        .returning(status.as_u16())?;
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

    // THEN it returns the status code
    assert_that!(res)
        .is_ok()
        .matches(|res| res.status() == status);
    connector.checkpoint()?;

    Ok(())
}

#[rstest]
#[case(StatusCode::ACCEPTED)]
#[case(StatusCode::SEE_OTHER)]
#[case(StatusCode::NOT_FOUND)]
#[case(StatusCode::NOT_ACCEPTABLE)]
#[case(StatusCode::INTERNAL_SERVER_ERROR)]
#[tokio::test]
async fn test_status_tuple(
    #[case] status: StatusCode,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a client that returns `status` as a tuple
    let mut builder = Connector::builder();
    builder
        .expect()
        .times(1)
        .with_uri("http://test.example")
        .returning((status.as_u16(), "moved"))?;
    let connector = builder.build();

    let client = client(connector.clone());

    // WHEN making a requests
    let res = client
        .request(
            Request::builder()
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns the status code
    assert_that!(res)
        .is_ok()
        .matches(|res| res.status() == status);
    connector.checkpoint()?;

    Ok(())
}

#[rstest]
#[case(StatusCode::ACCEPTED)]
#[case(StatusCode::SEE_OTHER)]
#[case(StatusCode::NOT_FOUND)]
#[case(StatusCode::NOT_ACCEPTABLE)]
#[case(StatusCode::INTERNAL_SERVER_ERROR)]
#[tokio::test]
async fn test_status_fn(#[case] status: StatusCode) -> Result<(), Box<dyn StdError + Send + Sync>> {
    // GIVEN a client that returns `status` through a closure
    let mut builder = Connector::builder();
    let moved_status = status.clone();
    builder
        .expect()
        .times(1)
        .with_uri("http://test.example")
        .returning(move |_| async move {
            Response::builder()
                .status(moved_status)
                .header("location", "/some-location")
                .body("")
        })?;
    let connector = builder.build();

    let client = client(connector.clone());

    // WHEN making a requests
    let res = client
        .request(
            Request::builder()
                .uri("http://test.example")
                .body("".to_string().into())?,
        )
        .await;

    // THEN it returns the status code
    assert_that!(res)
        .is_ok()
        .matches(|res| res.status() == status);
    connector.checkpoint()?;

    Ok(())
}
